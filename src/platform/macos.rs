use objc::rc::StrongPtr;

use crate::*;

pub trait WebViewExtMacOS {
    /// Get a strong pointer to the internal WKWebView object
    fn wk_web_view(&self) -> StrongPtr;
}

impl WebViewExtMacOS for WebView {
    fn wk_web_view(&self) -> StrongPtr {
        self.platform.web_view.clone()
    }
}

pub trait WebViewBuilderExtMacOS {
    fn with_debug(self, enable: bool) -> Self;
}

impl<T: request::RequestHandler> WebViewBuilderExtMacOS for WebViewBuilder<T> {
    fn with_debug(mut self, enable: bool) -> Self {
        self.platform.enable_debug = enable;
        self
    }
}
