extern crate base64;
extern crate hmac;
extern crate percent_encoding;
extern crate reqwest;
extern crate sha1;
extern crate uuid;

use self::hmac::{Hmac, Mac};
use self::percent_encoding::{utf8_percent_encode, EncodeSet};
use self::reqwest::Url;
use self::sha1::Sha1;
use self::uuid::Uuid;
use error::FlickrError;
use std::time::SystemTime;
use std::io;

type HmacSha1 = Hmac<Sha1>;
type FlickrResult<T> = Result<T, FlickrError>;

#[derive(Copy, Clone, Debug)]
#[allow(non_camel_case_types)]
struct UNRESERVED_ENCODE_SET;

impl EncodeSet for UNRESERVED_ENCODE_SET {
    fn contains(&self, byte: u8) -> bool {
        if byte.is_ascii_lowercase() || byte.is_ascii_uppercase() || byte.is_ascii_digit() {
            return false;
        }

        match byte {
            b'-' | b'.' | b'_' | b'~' => false,
            _ => true,
        }
    }
}

pub enum Stat {
    Ok,
    Fail,
}

pub struct PhotoRaw {
    title: String,
    ispublic: u32,
    url_k: String, // TODO: Add serde Url crate
    height_k: u32,
    width_k: String,
}

pub struct Photo {
    title: String,
    ispublic: bool,
    url_k: String, // TODO: Add serde Url crate
    height_k: u32,
    width_k: u32,
}

pub struct PhotosResponse {
    photo: Vec<PhotoRaw>,
    stat: Stat,
}

pub struct User {
    nsid: String,
    username: String,
    fullname: String,
}

pub struct ConsumerKey(pub String);
pub struct ConsumerSecret(pub String);
#[derive(Debug)]
pub struct TokenSecret(String);
#[derive(Debug)]
pub struct AccessToken {
    token: String,
    secret: TokenSecret,
}
pub struct RequestToken {
    token: String,
    secret: TokenSecret,
}

impl Default for TokenSecret {
    fn default() -> Self {
        TokenSecret(String::from(""))
    }
}

pub fn check_token(access_token: &AccessToken) -> User {
    unimplemented!()
}

/// Perform an OAuth 1.0 authentication flow to obtain an access token
pub fn authenticate(consumer_key: &ConsumerKey, consumer_secret: &ConsumerSecret) -> FlickrResult<AccessToken> {
    let request_token = get_request_token(consumer_key, consumer_secret)?;
    {
        let authorization_params = [
            ("oauth_token", request_token.token.as_str()),
            ("perms", "read"),
        ];
        let authorization_url = Url::parse_with_params("https://www.flickr.com/services/oauth/authorize", &authorization_params).expect("unable to parse authorization_url");

        println!("Visit this url in your browser to authorize the application:\n\n{}", authorization_url);
    }

    let mut verification_code = String::new();
    while verification_code.trim().is_empty() {
        print!("\nEnter the code: ");
        io::stdin().read_line(&mut verification_code).map_err(|_err| FlickrError::AuthenticationError)?;
    }

    // Exchange request token for access token
    exchange_request_token(request_token, verification_code.trim(), consumer_key, consumer_secret)
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

fn escape(value: &str) -> String {
    // Percent encode according to OAuth requirements
    utf8_percent_encode(value, UNRESERVED_ENCODE_SET).collect::<String>()
}

fn sign(base_string: &str, consumer_secret: &ConsumerSecret, token_secret: &TokenSecret) -> String {
    let key = format!("{}&{}", escape(&consumer_secret.0), escape(&token_secret.0));
    let mut mac = HmacSha1::new_varkey(key.as_bytes()).expect("Unable to create HMACer");
    mac.input(base_string.as_bytes());

    // oauth_signature is set to the calculated digest octet string, first base64-encoded per
    // [RFC2045] section 6.8, then URL-encoded
    base64::encode(&mac.result().code())
}

fn signature_base_string(
    verb: reqwest::Method,
    base_url: &str,
    params: &[(&str, String)],
) -> String {
    let mut params = params
        .iter()
        .map(|(k, v)| format!("{}={}", escape(k), escape(v)))
        .collect::<Vec<_>>();
    params.sort();

    format!(
        "{}&{}&{}",
        verb.to_string(),
        escape(base_url),
        escape(&params.join("&")),
    )
}

fn signature(verb: reqwest::Method, base_url: &str, params: &[(&str, String)], consumer_secret: &ConsumerSecret, token_secret: &TokenSecret) -> String {
    let base_string = signature_base_string(verb, base_url, params);
    sign(
        &base_string,
        consumer_secret,
        token_secret,
    )
}

fn parse_oauth_response(body: &str) -> Vec<(String, String)> {
    body.split("&")
        .map(|pair| {
            let mut iter = pair.split("=");
            match (iter.next(), iter.next()) {
                (Some(key), Some(value)) => (key.to_string(), value.to_string()),
                _ => panic!("malformed OAuth response")
            }
        }).collect()
}

fn get_request_token(consumer_key: &ConsumerKey, consumer_secret: &ConsumerSecret) -> FlickrResult<RequestToken> {
    let ConsumerKey(consumer_key) = consumer_key;

    let mut params = vec![
        ("oauth_nonce", generate_nonce()),
        ("oauth_timestamp", timestamp().to_string()),
        ("oauth_consumer_key", consumer_key.to_string()),
        ("oauth_signature_method", String::from("HMAC-SHA1")),
        ("oauth_version", String::from("1.0")),
        ("oauth_callback", String::from("oob")),
    ];

    let url = "https://www.flickr.com/services/oauth/request_token";
    let oauth_signature = signature(reqwest::Method::Get, url, &params, consumer_secret, &TokenSecret::default());
    params.push(("oauth_signature", oauth_signature));

    let url = Url::parse_with_params(url, params).expect("Unable to parse url");

    let mut res = reqwest::get(url).expect("error requesting request token");
    let body = res.text().unwrap();
    println!("{}", body);

    // oauth_callback_confirmed=true&oauth_token=xxxxxx&oauth_token_secret=xxxxxx
    let mut token = None;
    let mut secret = None;
    for (key, value) in parse_oauth_response(&body) {
        match key.as_str() {
            "oauth_token" => token = Some(value),
            "oauth_token_secret" => secret = Some(value),
            _ => ()
        }
    }

    match (token, secret) {
        (Some(token), Some(secret)) => Ok(RequestToken { token, secret: TokenSecret(secret) }),
        _ => Err(FlickrError::AuthenticationError)
    }
}

fn exchange_request_token(request_token: RequestToken, verification_code: &str, consumer_key: &ConsumerKey, consumer_secret: &ConsumerSecret) -> FlickrResult<AccessToken> {
    let ConsumerKey(consumer_key) = consumer_key;

    let mut params = vec![
        ("oauth_nonce", generate_nonce()),
        ("oauth_timestamp", timestamp().to_string()),
        ("oauth_token", request_token.token),
        ("oauth_verifier", verification_code.to_string()),
        ("oauth_consumer_key", consumer_key.to_string()),
        ("oauth_signature_method", String::from("HMAC-SHA1")),
        ("oauth_version", String::from("1.0")),
    ];

    let url = "https://www.flickr.com/services/oauth/access_token";
    let oauth_signature = signature(reqwest::Method::Get, url, &params, consumer_secret, &request_token.secret);
    params.push(("oauth_signature", oauth_signature));

    let url = Url::parse_with_params(url, params).expect("Unable to parse url");

    let mut res = reqwest::get(url).expect("error requesting request token");
    let body = res.text().unwrap();
    println!("{}", body);

    // Flickr returns a response similar to the following:
    // fullname=Jamal%20Fanaian
    // &oauth_token=72157626318069415-087bfc7b5816092c
    // &oauth_token_secret=a202d1f853ec69de
    // &user_nsid=21207597%40N07
    // &username=jamalfanaian
    let mut token = None;
    let mut secret = None;
    for (key, value) in parse_oauth_response(&body) {
        match key.as_str() {
            "oauth_token" => token = Some(value),
            "oauth_token_secret" => secret = Some(value),
            _ => ()
        }
    }

    match (token, secret) {
        (Some(token), Some(secret)) => Ok(AccessToken { token, secret: TokenSecret(secret) }),
        _ => Err(FlickrError::AuthenticationError)
    }
}

#[test]
fn test_signature_base_string() {
    // Verify example from https://www.flickr.com/services/api/auth.oauth.html
    let params = [
        ("oauth_nonce", String::from("95613465")),
        ("oauth_timestamp", String::from("1305586162")),
        (
            "oauth_consumer_key",
            String::from("653e7a6ecc1d528c516cc8f92cf98611"),
        ),
        ("oauth_signature_method", String::from("HMAC-SHA1")),
        ("oauth_version", String::from("1.0")),
        ("oauth_callback", String::from("http://www.example.com")),
    ];

    assert_eq!(signature_base_string(reqwest::Method::Get, "https://www.flickr.com/services/oauth/request_token", &params), "GET&https%3A%2F%2Fwww.flickr.com%2Fservices%2Foauth%2Frequest_token&oauth_callback%3Dhttp%253A%252F%252Fwww.example.com%26oauth_consumer_key%3D653e7a6ecc1d528c516cc8f92cf98611%26oauth_nonce%3D95613465%26oauth_signature_method%3DHMAC-SHA1%26oauth_timestamp%3D1305586162%26oauth_version%3D1.0");
}

// #[test]
// fn test_sign() {
//     // This verifies the example from https://www.flickr.com/services/api/auth.oauth.html
//     assert_eq!(sign("GET&https%3A%2F%2Fwww.flickr.com%2Fservices%2Foauth%2Frequest_token&oauth_callback%3Dhttp%253A%252F%252Fwww.example.com%26oauth_consumer_key%3D653e7a6ecc1d528c516cc8f92cf98611%26oauth_nonce%3D95613465%26oauth_signature_method%3DHMAC-SHA1%26oauth_timestamp%3D1305586162%26oauth_version%3D1.0"), "7w18YS2bONDPL/zgyzP5XTr5af4=");
// }
