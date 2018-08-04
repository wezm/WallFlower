extern crate reqwest;
extern crate serde_json;

use std::{io, num};

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
    #[fail(display = "URL error")]
    UrlError(reqwest::UrlError),
    #[fail(display = "Parse error")]
    ParseError(num::ParseIntError),
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

impl From<reqwest::UrlError> for FlickrError {
    fn from(err: reqwest::UrlError) -> Self {
        FlickrError::UrlError(err)
    }
}

impl From<num::ParseIntError> for FlickrError {
    fn from(err: num::ParseIntError) -> Self {
        FlickrError::ParseError(err)
    }
}
