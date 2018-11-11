use reqwest::{self, Url};
use threadpool::ThreadPool;
use piston::window::{Size};
use opengl_graphics::*;
use image::DynamicImage;
use percent_encoding;
use serde_json;

use std;
use std::borrow::Borrow;
use std::fs::{self, File};
use std::io;
use std::path::{Path, PathBuf};
use std::sync::mpsc::channel;
use std::ffi::OsStr;
use image::{self, Pixel, Rgba};
use graphics::color::gamma_srgb_to_linear;

use flickr::{self, AccessToken, AuthenticatedClient, Photo};
use WallflowerError;

enum Dimension {
    Width(u32),
    Height(u32),
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
    let filename = path.file_name().ok_or_else(|| {
        WallflowerError::IoError(io::Error::new(
            io::ErrorKind::Other,
            "URL does not have file name",
        ))
    })?;

    // Check if photo has already been downloaded
    let mut storage_path = PathBuf::new();
    storage_path.push("photos");
    storage_path.push(filename);

    if storage_path.is_file() {
        println!("{} -> exists", url);
        Ok(())
    } else {
        // download the file
        println!("{} -> downloading", url);
        download_file(url, &storage_path)
    }
}

fn fetch_photo(photo: Photo, tx: std::sync::mpsc::Sender<Result<(), WallflowerError>>) {
    tx.send(do_fetch_photo(&photo.url_k))
        .expect("error sending to channel");
}

pub fn update_photostream(user_id: &str, client: &AuthenticatedClient) -> Result<(), WallflowerError> {
    // Request list of photos from Flickr
    // Download the ones that aren't in the cache
    // (optional) Clean up old images
    // Generate new JSON, move into place atomically

    let pool = ThreadPool::new(8);
    let (tx, rx) = channel();

    // Check the last 500 photos
    // TODO: photos page="2" pages="89" perpage="10" total="881">
    // Stop if there are fewer than 5 pages
    for page in 1..3 {
        let arguments = [
            ("min_taken_date", "1388494800".to_string()),
            ("content_type", "1".to_string()), // Photos only
            ("per_page", "100".to_string()),
            ("page", page.to_string()),
            ("extras", "url_k".to_string()),
        ];
        let photos = client.photos(user_id, &arguments)?;

        //println!("{:?}", photos);

        let photo_count = photos.len();

        for photo in photos {
            let tx = tx.clone();
            pool.execute(move || fetch_photo(photo, tx))
        }

        rx.iter().take(photo_count).for_each(|result| {
            if result.is_err() {
                println!("{:?}", result)
            }
        });
    }

    Ok(())
}

pub fn load_access_token<P: AsRef<Path>>(client: flickr::Client, path: P) -> Result<AuthenticatedClient, WallflowerError> {
    match File::open(path.as_ref()) {
        Ok(file) => {
            let access_token: AccessToken = serde_json::from_reader(file)?;
            Ok(AuthenticatedClient::new(client, access_token))
        }
        Err(e) => {
            println!("{:?}", e);
            let client = client.authenticate()?;

            // Save app data for using on the next run.
            let file = File::create(path.as_ref())?;
            let _ = serde_json::to_writer_pretty(file, client.access_token())?;

            Ok(client)
        }
    }
}

fn largest_dimension(size: Size) -> Dimension {
    if size.width > size.height {
        Dimension::Width(size.width)
    } else {
        Dimension::Height(size.height)
    }
}

pub fn zoom_for_image(window_size: Size, image_size: Size) -> f64 {
    match largest_dimension(image_size) {
        Dimension::Width(width) => window_size.width as f64 / width as f64,
        Dimension::Height(height) => window_size.height as f64 / height as f64,
    }
}

pub fn translation_for_image(window_width: u32, image_width: f64) -> f64 {
    (window_width as f64 / 2.) - (image_width / 2.)
}

pub fn load_photo<P: AsRef<Path>>(path: P) -> Result<Texture, WallflowerError> {
    println!("loading {:?}", path.as_ref());

    let photo = image::open(&path).map_err(|_err| {
        println!("{:?}", _err);
        WallflowerError::GraphicsError
    })?;

    let photo = match photo {
        DynamicImage::ImageRgba8(photo) => photo,
        x => x.to_rgba(),
    };

    let photo = convert_image_from_srgb_to_linear(photo);
    Ok(Texture::from_image(&photo, &TextureSettings::new().convert_gamma(true)))
}

// Source: https://github.com/Rydgel/rust-rogue/blob/b09400daec0a84a82d6b357e9ffa5f55c68afd5c/src/drawings/sprites.rs#L64
// Licence: MIT Copyright (c) 2016 Jérôme Mahuet
fn convert_image_from_srgb_to_linear(img: image::ImageBuffer<Rgba<u8>, Vec<u8>>) -> image::ImageBuffer<Rgba<u8>, Vec<u8>> {
    let mut new_img = img.clone();

    for (x, y, pixel) in img.enumerate_pixels() {
        let (r, g, b, a) = pixel.channels4();
        let r = r as f32 / 255.0;
        let g = g as f32 / 255.0;
        let b = b as f32 / 255.0;
        let a = a as f32 / 255.0;
        let new_color = gamma_srgb_to_linear([r, g, b, a]);
        let r = (new_color[0] * 255.0) as u8;
        let g = (new_color[1] * 255.0) as u8;
        let b = (new_color[2] * 255.0) as u8;
        let a = (new_color[3] * 255.0) as u8;
        let new_pixel = image::Pixel::from_channels(r, g, b, a);
        new_img.put_pixel(x, y, new_pixel);
    }

    new_img
}

pub fn available_photos(dir: &str) -> Result<Vec<PathBuf>, WallflowerError> {
    let mut photos = vec![];
    let jpg = OsStr::new("jpg");

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension()
            .map(|ext| ext == jpg && path.is_file())
            .unwrap_or(false)
        {
            println!("adding {:?}", path);
            photos.push(path);
        }
    }

    Ok(photos)
}

