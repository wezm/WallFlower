extern crate serde_json;

use std::io;

#[derive(Fail, Debug)]
pub enum FlickrError {
    #[fail(display = "The request was rejected")]
    AuthenticationError,
    #[fail(display = "JSON error")]
    JsonError(serde_json::Error),
    #[fail(display = "I/O error")]
    IoError(io::Error),
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
