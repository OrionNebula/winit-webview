use std::io;

pub use crate::platform_impl::PlatformRequest;

pub struct Response<T: io::Read> {
    pub body: T,
    pub mime_type: String
}

pub trait RequestHandler {
    type Read: io::Read;

    /// Handle a request to the specified URI
    fn handle_request(&mut self, uri: &str) -> Option<Response<Self::Read>>;

    /// Handle a request in its platform-specific form
    fn handle_platform_request(&mut self, request: PlatformRequest) -> Option<Response<Self::Read>> {
        self.handle_request(request.as_uri())
    }
}

/// A request handler that always 404s
pub struct NullRequestHandler;

impl RequestHandler for NullRequestHandler {
    type Read = io::Empty;

    fn handle_request(&mut self, _uri: &str) -> Option<Response<Self::Read>> {
        None
    }
}

