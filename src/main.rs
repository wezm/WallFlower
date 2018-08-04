extern crate actix;
extern crate actix_web;
extern crate bytes;
extern crate env_logger;
extern crate futures;
extern crate hyper;
extern crate reqwest;
extern crate percent_encoding;
extern crate serde_json;
extern crate threadpool;
extern crate tokio_core;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate json;
use threadpool::ThreadPool;
use std::sync::mpsc::channel;
extern crate wallflower;

use actix_web::{
    error, http, middleware, server, App, AsyncResponder, Error, HttpMessage, HttpRequest,
    HttpResponse, Json,
};

use bytes::BytesMut;
use futures::{future::ok, Future, Stream};
use hyper::Client;
use json::JsonValue;
use std::fs::File;
use std::env;
use std::io;
use std::path::{Path, PathBuf};
use std::borrow::Borrow;
use tokio_core::reactor::Core;
use reqwest::Url;

use wallflower::flickr::{self, AccessToken, AuthenticatedClient, Photo};
use wallflower::WallflowerError;

#[derive(Debug, Serialize, Deserialize)]
struct MyObj {
    name: String,
    number: i32,
}

/// This handler uses `HttpRequest::json()` for loading json object.
fn index(req: &HttpRequest) -> Box<Future<Item = HttpResponse, Error = Error>> {
    req.json()
        .from_err()  // convert all errors into `Error`
        .and_then(|val: MyObj| {
            println!("model: {:?}", val);
            Ok(HttpResponse::Ok().json(val))  // <- send response
        })
        .responder()
}

/// This handler uses json extractor
fn extract_item(item: Json<MyObj>) -> HttpResponse {
    println!("model: {:?}", &item);
    HttpResponse::Ok().json(item.0) // <- send response
}

/// This handler uses json extractor with limit
fn extract_item_limit((item, _req): (Json<MyObj>, HttpRequest)) -> HttpResponse {
    println!("model: {:?}", &item);
    HttpResponse::Ok().json(item.0) // <- send response
}

const MAX_SIZE: usize = 262_144; // max payload size is 256k

/// This handler manually load request payload and parse json object
fn index_manual(req: &HttpRequest) -> Box<Future<Item = HttpResponse, Error = Error>> {
    // HttpRequest::payload() is stream of Bytes objects
    req.payload()
        // `Future::from_err` acts like `?` in that it coerces the error type from
        // the future into the final error type
        .from_err()

        // `fold` will asynchronously read each chunk of the request body and
        // call supplied closure, then it resolves to result of closure
        .fold(BytesMut::new(), move |mut body, chunk| {
            // limit max size of in-memory payload
            if (body.len() + chunk.len()) > MAX_SIZE {
                Err(error::ErrorBadRequest("overflow"))
            } else {
                body.extend_from_slice(&chunk);
                Ok(body)
            }
        })
        // `Future::and_then` can be used to merge an asynchronous workflow with a
        // synchronous workflow
        .and_then(|body| {
            // body is loaded, now we can deserialize serde-json
            let obj = serde_json::from_slice::<MyObj>(&body)?;
            Ok(HttpResponse::Ok().json(obj)) // <- send response
        })
        .responder()
}

/// This handler manually load request payload and parse json-rust
fn index_mjsonrust(req: &HttpRequest) -> Box<Future<Item = HttpResponse, Error = Error>> {
    req.payload()
        .concat2()
        .from_err()
        .and_then(|body| {
            // body is loaded, now we can deserialize json-rust
            let result = json::parse(std::str::from_utf8(&body).unwrap()); // return Result
            let injson: JsonValue = match result {
                Ok(v) => v,
                Err(e) => object!{"err" => e.to_string() },
            };
            Ok(HttpResponse::Ok()
                .content_type("application/json")
                .body(injson.dump()))
        })
        .responder()
}

fn download_file(url: &Url, path: &Path) -> Result<(), WallflowerError> {
    let mut file = File::create(path)?;
    // TODO: Check that content type suggests it's actually an image
    // FIXME: reqwest::get creates a new client for each request. Ideally each thread would have its own client and that would be reused for each request that worker serviced
    reqwest::get(url.as_str())?.copy_to(&mut file)?;

    Ok(())
}

fn do_fetch_photo(url: &Url) -> Result<(), WallflowerError> {
    // let path = Path::new("photos");
    let percent_encoded_path = url.path();
    let cow = percent_encoding::percent_decode(percent_encoded_path.as_bytes()).decode_utf8()?;
    let path: &str = cow.borrow();
    let path = Path::new(path);
    let filename = path.file_name().ok_or_else(|| WallflowerError::IoError(io::Error::new(io::ErrorKind::Other, "URL does not have file name")))?;

    // Check if photo has already been downloaded
    let mut storage_path = PathBuf::new();
    storage_path.push("photos");
    storage_path.push(filename);

    if storage_path.is_file() {
        println!("{} -> exists", url);
        Ok(())
    }
    else {
        // download the file
        println!("{} -> downloading", url);
        download_file(url, &storage_path)
    }
}

fn fetch_photo(photo: Photo, tx: std::sync::mpsc::Sender<Result<(), WallflowerError>>) {
    tx.send(do_fetch_photo(&photo.url_k));
}

fn update_photostream(user_id: &str, client: &AuthenticatedClient) -> Result<(), WallflowerError> {
    // Request list of photos from Flickr
    // Download the ones that aren't in the cache
    // (optional) Clean up old images
    // Generate new JSON, move into place atomically

    // let url = format!("https://api.flickr.com/services/rest/?method=flickr.people.getPhotos&api_key={api_key}&format=json&nojsoncallback=1&user_id={user_id}&min_taken_date=1388494800&content_type=1&privacy_filter=5&per_page=100&extras=url_k", api_key = API_KEY, user_id = USER_ID).parse().unwrap();
    let arguments = [
        ("min_taken_date", "1388494800".to_string()),
        ("content_type", "1".to_string()), // Photos only
        ("per_page", "100".to_string()),
        ("extras", "url_k".to_string()),
    ];
    let photos = client.photos(user_id, &arguments)?;

    println!("{:?}", photos);

    let pool = ThreadPool::new(8);
    let (tx, rx) = channel();
    let photo_count = photos.len();

    for photo in photos {
        let tx = tx.clone();
        pool.execute(move || fetch_photo(photo, tx))
    }

    rx.iter().take(photo_count).for_each(|result| println!("{:?}", result));

    Ok(())
}

const FLICKR_DATA_FILE: &str = ".flickr-data.json";

fn load_access_token(client: flickr::Client) -> Result<AuthenticatedClient, WallflowerError> {
    match File::open(FLICKR_DATA_FILE) {
        Ok(file) => {
            let access_token: AccessToken = serde_json::from_reader(file)?;
            Ok(AuthenticatedClient::new(client, access_token))
        }
        Err(e) => {
            println!("{:?}", e);
            let client = client.authenticate()?;

            // Save app data for using on the next run.
            let file = File::create(FLICKR_DATA_FILE)?;
            let _ = serde_json::to_writer_pretty(file, client.access_token())?;

            Ok(client)
        }
    }
}

fn main() -> Result<(), WallflowerError> {
    env_logger::init();

    let api_key = env::var("FLICKR_API_KEY").expect("FLICKR_API_KEY must be set");
    let api_secret = env::var("FLICKR_API_SECRET").expect("FLICKR_API_SECRET must be set");

    let client = flickr::Client::new(&api_key, &api_secret);
    let client = load_access_token(client)?;

    // Verify token, and get user info
    let token_info = client.check_token()?;

    println!("{:?}", token_info);

    update_photostream(&token_info.user.nsid, &client)?;

    Ok(())
}

fn main2() {
    ::std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();
    let sys = actix::System::new("json-example");

    server::new(|| {
        App::new()
            // enable logger
            .middleware(middleware::Logger::default())
            .resource("/extractor", |r| {
                r.method(http::Method::POST)
                    .with_config(extract_item, |cfg| {
                        cfg.limit(4096); // <- limit size of the payload
                    })
            })
            .resource("/extractor2", |r| {
                r.method(http::Method::POST)
                    .with_config(extract_item_limit, |cfg| {
                        cfg.0.limit(4096); // <- limit size of the payload
                    })
            })
            .resource("/manual", |r| r.method(http::Method::POST).f(index_manual))
            .resource("/mjsonrust", |r| r.method(http::Method::POST).f(index_mjsonrust))
            .resource("/", |r| r.method(http::Method::POST).f(index))
    }).bind("127.0.0.1:8080")
        .unwrap()
        .shutdown_timeout(1)
        .start();

    println!("Started http server: 127.0.0.1:8080");
    let _ = sys.run();
}
