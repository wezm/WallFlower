extern crate serde_json;
extern crate reqwest;

use std::io;

#[derive(Fail, Debug)]
pub enum FlickrError {
    #[fail(display = "The request was rejected")]
    AuthenticationError,
    #[fail(display = "JSON error")]
    JsonError(serde_json::Error),
    #[fail(display = "I/O error")]
    IoError(io::Error),
    #[fail(display = "HTTP error")]
    HttpError(reqwest::Error),
}

impl From<serde_json::Error> for FlickrError {
    fn from(err: serde_json::Error) -> Self {
        FlickrError::JsonError(err)
    }
}

impl From<io::Error> for FlickrError {
    fn from(err: io::Error) -> Self {
        FlickrError::IoError(err)
    }
}

impl From<reqwest::Error> for FlickrError {
    fn from(err: reqwest::Error) -> Self {
        FlickrError::HttpError(err)
    }
}
