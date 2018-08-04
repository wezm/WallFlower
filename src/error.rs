#[derive(Fail, Debug)]
pub enum FlickrError {
    #[fail(display = "The request was rejected")]
    AuthenticationError,
}
