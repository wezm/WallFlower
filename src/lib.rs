extern crate failure;
#[macro_use]
extern crate failure_derive;
#[macro_use]
extern crate serde_derive;
extern crate chrono;
extern crate env_logger;
extern crate glfw_window;
extern crate graphics;
extern crate image;
extern crate opengl_graphics;
extern crate percent_encoding;
extern crate piston;
extern crate reqwest;
extern crate serde_json;
extern crate threadpool;

pub mod error;
pub mod flickr;
pub mod weather;
pub mod slideshow;
pub mod statusbar;

pub use error::FlickrError;
pub use error::WallflowerError;
