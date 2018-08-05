extern crate reqwest;
extern crate serde_json;

use std::{io, num, str};

#[derive(Fail, Debug)]
pub enum WallflowerError {
    #[fail(display = "I/O error")]
    IoError(io::Error),
    #[fail(display = "HTTP error")]
    HttpError(reqwest::Error),
    #[fail(display = "UTF-8 parse error")]
    ParseError(str::Utf8Error),
    #[fail(display = "Flickr error")]
    FlickrError(FlickrError),
    #[fail(display = "JSON error")]
    JsonError(serde_json::Error),
    #[fail(display = "Graphics error")]
    GraphicsError,
}

impl From<str::Utf8Error> for WallflowerError {
    fn from(err: str::Utf8Error) -> Self {
        WallflowerError::ParseError(err)
    }
}

impl From<FlickrError> for WallflowerError {
    fn from(err: FlickrError) -> Self {
        WallflowerError::FlickrError(err)
    }
}

impl From<serde_json::Error> for WallflowerError {
    fn from(err: serde_json::Error) -> Self {
        WallflowerError::JsonError(err)
    }
}

impl From<io::Error> for WallflowerError {
    fn from(err: io::Error) -> Self {
        WallflowerError::IoError(err)
    }
}

impl From<reqwest::Error> for WallflowerError {
    fn from(err: reqwest::Error) -> Self {
        WallflowerError::HttpError(err)
    }
}

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
