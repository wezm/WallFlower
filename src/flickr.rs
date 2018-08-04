extern crate reqwest;
extern crate uuid;
extern crate hmac;
extern crate sha1;
extern crate base64;

use std::time::SystemTime;
use error::FlickrError;
use self::reqwest::Url;
use self::uuid::Uuid;
use self::hmac::{Hmac, Mac};
use self::sha1::Sha1;

type HmacSha1 = Hmac<Sha1>;
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
    let request_token = get_request_token(consumer_key);

    Ok(AccessToken(String::from("access_token")))
}

fn generate_nonce() -> String {
    Uuid::new_v4().to_string()
}

fn timestamp() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("SystemTime before UNIX_EPOCH")
        .as_secs()
}

fn sign(base_string: &str) -> String {
    let mut mac = HmacSha1::new_varkey(b"FIXME&") // FIXME
        .expect("Unable to create HMACer");
    mac.input(base_string.as_bytes());

    // oauth_signature is set to the calculated digest octet string, first base64-encoded per
    // [RFC2045] section 6.8, then URL-encoded
    base64::encode(&mac.result().code())
}

fn signature(verb: reqwest::Method, base_url: &str, params: &[(&str, String)]) -> String {
    let mut params = params.to_vec();
    params.sort_by(|a, b| a.0.cmp(b.0));

    let url = Url::parse_with_params(base_url, params).expect("Unable to parse url");

    sign(&[verb.to_string(), url.to_string()].join("&"))
}

fn get_request_token(consumer_key: &ConsumerKey) -> FlickrResult<RequestToken> {
    let timestamp = timestamp().to_string();
    let nonce = generate_nonce();
    let ConsumerKey(consumer_key) = consumer_key;

    let mut params = vec![
        ("oauth_nonce", nonce),
        ("oauth_timestamp", timestamp),
        ("oauth_consumer_key", consumer_key.to_string()),
        ("oauth_signature_method", String::from("HMAC-SHA1")),
        ("oauth_version", String::from("1.0")),
        ("oauth_callback", String::from("oob")),
    ];

    let base_url = "https://www.flickr.com/services/oauth/request_token";
    let oauth_signature = signature(reqwest::Method::Get, base_url, &params);
    params.push(("oauth_signature", oauth_signature));

    let url = Url::parse_with_params(base_url, params).expect("Unable to parse url");

    let mut res = reqwest::get(url).expect("error requesting request token");
    println!("{}", res.text().unwrap());

    Ok(RequestToken(String::from("nope")))
}

fn exchange_request_token(request_token: RequestToken) -> FlickrResult<AccessToken> {
    unimplemented!()
}

#[test]
fn test_sign() {
    // This verifies the example from https://www.flickr.com/services/api/auth.oauth.html
    assert_eq!(sign("GET&https%3A%2F%2Fwww.flickr.com%2Fservices%2Foauth%2Frequest_token&oauth_callback%3Dhttp%253A%252F%252Fwww.example.com%26oauth_consumer_key%3D653e7a6ecc1d528c516cc8f92cf98611%26oauth_nonce%3D95613465%26oauth_signature_method%3DHMAC-SHA1%26oauth_timestamp%3D1305586162%26oauth_version%3D1.0"), "7w18YS2bONDPL/zgyzP5XTr5af4=");
}
