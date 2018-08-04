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

#[derive(Debug, Deserialize)]
pub struct User {
    nsid: String,
    username: String,
    fullname: String,
}

#[derive(Debug, Deserialize)]
pub struct OauthToken {
    token: String,
    perms: String,
    user: User,
}

impl From<CheckTokenResponse> for OauthToken {
    fn from(res: CheckTokenResponse) -> Self {
        OauthToken {
            token: res.oauth.token.content,
            perms: res.oauth.perms.content,
            user: res.oauth.user
        }
    }
}

#[derive(Debug, Deserialize)]
struct Element {
    #[serde(rename="_content")]
    content: String,
}

#[derive(Debug, Deserialize)]
struct CheckTokenResponseOauth {
    token: Element,
    perms: Element,
    user: User
}

// {"oauth":{"token":{"_content":"72157698177686331-fc8f8f2c03d4fb0d"},"perms":{"_content":"read"},"user":{"nsid":"40215689@N00","username":"wezm","fullname":"Wesley Moore"}},"stat":"ok"}
#[derive(Debug, Deserialize)]
struct CheckTokenResponse {
    oauth: CheckTokenResponseOauth
}

#[derive(Debug)]
pub struct ConsumerKey(pub String);
#[derive(Debug)]
pub struct ConsumerSecret(pub String);
#[derive(Debug, Serialize, Deserialize)]
pub struct TokenSecret(String);
#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug)]
pub struct Client {
    consumer_key: ConsumerKey,
    consumer_secret: ConsumerSecret,
}

#[derive(Debug)]
pub struct AuthenticatedClient {
    consumer_key: ConsumerKey,
    consumer_secret: ConsumerSecret,
    access_token: AccessToken,
}

impl Client {
    pub fn new(consumer_key: &str, consumer_secret: &str) -> Self {
        Client { consumer_key: ConsumerKey(consumer_key.to_string()), consumer_secret: ConsumerSecret(consumer_secret.to_string()) }
    }

    /// Perform an OAuth 1.0 authentication flow to obtain an access token
    pub fn authenticate(self) -> FlickrResult<AuthenticatedClient> {
        let request_token = self.get_request_token()?;
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
        let access_token = self.exchange_request_token(request_token, verification_code.trim())?;

        Ok(AuthenticatedClient {
            consumer_key: self.consumer_key,
            consumer_secret: self.consumer_secret,
            access_token
        })
    }

    fn get_request_token(&self) -> FlickrResult<RequestToken> {
        let ConsumerKey(ref consumer_key) = self.consumer_key;

        let mut params = vec![
            ("oauth_nonce", generate_nonce()),
            ("oauth_timestamp", timestamp().to_string()),
            ("oauth_consumer_key", consumer_key.to_string()),
            ("oauth_signature_method", String::from("HMAC-SHA1")),
            ("oauth_version", String::from("1.0")),
            ("oauth_callback", String::from("oob")),
        ];

        let url = "https://www.flickr.com/services/oauth/request_token";
        let oauth_signature = signature(reqwest::Method::Get, url, &params, &self.consumer_secret, &TokenSecret::default());
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

    fn exchange_request_token(&self, request_token: RequestToken, verification_code: &str) -> FlickrResult<AccessToken> {
        let ConsumerKey(ref consumer_key) = self.consumer_key;

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
        let oauth_signature = signature(reqwest::Method::Get, url, &params, &self.consumer_secret, &request_token.secret);
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
}

impl AuthenticatedClient {
    pub fn new(client: Client, access_token: AccessToken) -> Self {
        AuthenticatedClient {
            consumer_key: client.consumer_key,
            consumer_secret: client.consumer_secret,
            access_token
        }
    }

    pub fn access_token(&self) -> &AccessToken {
        &self.access_token
    }

    pub fn check_token(&self) -> FlickrResult<OauthToken> {
        let ConsumerKey(ref consumer_key) = self.consumer_key;

        let method = String::from("flickr.auth.oauth.checkToken");

        let mut params = vec![
            ("oauth_nonce", generate_nonce()),
            ("oauth_timestamp", timestamp().to_string()),
            ("oauth_token", self.access_token.token.clone()),
            ("oauth_consumer_key", consumer_key.to_string()),
            ("oauth_signature_method", String::from("HMAC-SHA1")),
            ("oauth_version", String::from("1.0")),
            ("format", String::from("json")),
            ("nojsoncallback", String::from("1")),
            ("method", method.clone()),
        ];

        let url = "https://api.flickr.com/services/rest";
        let oauth_signature = signature(reqwest::Method::Get, url, &params, &self.consumer_secret, &self.access_token.secret);
        params.push(("oauth_signature", oauth_signature));

        let url = Url::parse_with_params(url, params).expect("Unable to parse url");

        let mut res = reqwest::get(url).expect(&format!("error calling {}", method));

        let token_res: CheckTokenResponse = res.json()?;

        Ok(token_res.into())
    }
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
