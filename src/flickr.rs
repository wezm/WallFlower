extern crate reqwest;

use std::time::SystemTime;
use error::FlickrError;
use self::reqwest::Url;

type FlickrResult<T> = Result<T, FlickrError>;

pub enum Stat {
    Ok,
    Fail
}

pub struct PhotoRaw {
    title: String,
    ispublic: u32,
    url_k: String, // TODO: Add serde Url crate
    height_k: u32,
    width_k: String
}

pub struct Photo {
    title: String,
    ispublic: bool,
    url_k: String, // TODO: Add serde Url crate
    height_k: u32,
    width_k: u32
}

pub struct PhotosResponse {
    photo: Vec<PhotoRaw>,
    stat: Stat
}

pub struct User {
    nsid: String,
    username: String,
    fullname: String
}

pub struct ConsumerKey(String);
pub struct AccessToken(String);
pub struct RequestToken(String);

pub fn check_token(access_token: &AccessToken) -> User {
    unimplemented!()
}

/// Perform an OAuth 1.0 authentication flow to obtain an access token
pub fn authenticate(consumer_key: &ConsumerKey) -> FlickrResult<AccessToken> {
    unimplemented!()
}

fn generate_nonce() -> String {
    unimplemented!()
}

fn timestamp() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("SystemTime before UNIX_EPOCH")
        .as_secs()
}

fn get_request_token() -> FlickrResult<RequestToken> {
    let timestamp = timestamp().to_string();

    let params = [
        ("oauth_nonce", "95613465"),
        ("oauth_timestamp", &timestamp),
        ("oauth_consumer_key", "653e7a6ecc1d528c516cc8f92cf98611"),
        ("oauth_signature_method", "HMAC-SHA1"),
        ("oauth_version", "1.0"),
        ("oauth_signature", "7w18YS2bONDPL%2FzgyzP5XTr5af4%3D"),
        ("oauth_callback", "http%3A%2F%2Fwww.example.com"),
    ];
    let url = Url::parse_with_params("https://www.flickr.com/services/oauth/request_token", &params).expect("Unable to parse url");

    Ok(RequestToken(String::from("nope")))
}

fn exchange_request_token(request_token: RequestToken) -> FlickrResult<AccessToken> {
    unimplemented!()
}
