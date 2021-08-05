#[cfg(target_os = "macos")]
extern crate core_graphics;
#[cfg(target_os = "macos")]
extern crate objc_foundation;
#[cfg(target_os = "macos")]
#[macro_use]
extern crate objc;
#[cfg(target_os = "macos")]
extern crate block;


#[macro_use]
extern crate paste;
extern crate winit;

mod events;
pub use events::*;

pub mod request;

pub mod platform;
mod platform_impl;

/// A builder for the WebView component.
pub struct WebViewBuilder<T: request::RequestHandler> {
    pub(crate) request_handler: T,
    pub(crate) init_scripts: Vec<String>,
    pub(crate) platform: platform_impl::PlatformWebViewBuilder
}

impl WebViewBuilder<request::NullRequestHandler> {
    /// Create a new WebViewBuilder
    pub fn new() -> Self {
        Self {
            init_scripts: Vec::new(),
            platform: platform_impl::PlatformWebViewBuilder::new(),
            request_handler: request::NullRequestHandler
        }
    }
}

impl<T: request::RequestHandler> WebViewBuilder<T> {
    pub fn with_request_handler(request_handler: T) -> Self {
        Self {
            init_scripts: Vec::new(),
            platform: platform_impl::PlatformWebViewBuilder::new(),
            request_handler
        }
    }

    /// Add a script to be injected into the page when it loads
    pub fn with_init_script(mut self, script: impl AsRef<str>) -> Self {
        self.init_scripts.push(script.as_ref().to_owned());
        self
    }

    /// Construct the WebView component
    pub fn build(self, event_handler: impl EventHandler, mut window: winit::window::Window) -> WebView {
        WebView {
            platform: platform_impl::PlatformWebView::build(self, event_handler, &mut window),
            window,
        }
    }
}

/// A handle to the WebView component
/// TODO: Upon dropping this handle, detach the webview from the winit window
pub struct WebView {
    pub window: winit::window::Window,
    platform: platform_impl::PlatformWebView
}

/// A target for navigation
pub enum NavigationTarget<'a> {
    Url(&'a str),
    Html(&'a str)
}

impl WebView {
    pub fn navigate(&mut self, target: NavigationTarget) {
        self.platform.navigate(target)
    }

    pub fn evaluate(&mut self, js: impl AsRef<str>) {
        self.platform.execute(js)
    }

    pub fn title(&self) -> Option<String> {
        self.platform.title()
    }
}
