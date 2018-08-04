extern crate failure;
#[macro_use]
extern crate failure_derive;
#[macro_use]
extern crate serde_derive;

pub mod error;
pub mod flickr;

pub use error::FlickrError;
