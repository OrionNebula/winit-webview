use crate::*;
use block::ConcreteBlock;
use core_graphics::display::CGRect;
use objc::{rc::StrongPtr, runtime::Object};
use objc_foundation::{INSData, INSString, NSData, NSString};
use std::ffi::c_void;
use winit::{platform::macos::WindowExtMacOS, window::Window};

#[macro_use]
mod macros;

#[derive(Debug, Default)]
pub struct PlatformWebViewBuilder {
    pub(crate) enable_debug: bool,
}

impl PlatformWebViewBuilder {
    pub fn new() -> Self {
        Default::default()
    }
}

#[derive(Debug, Clone, Copy)]
pub enum PlatformNavigationEvent {
    /// Triggered when a redirect has taken place
    Redirect,
}

#[derive(Debug, Clone, Copy)]
pub enum PlatformEvent {}

pub struct PlatformWebView {
    pub(crate) _delegate: StrongPtr,
    pub(crate) web_view: StrongPtr,
}

pub struct PlatformRequest<'a> {
    pub uri: &'a str
}

impl<'a> PlatformRequest<'a> {
    pub fn as_uri(&self) -> &str {
        self.uri
    }
}

impl PlatformWebView {
    pub fn build(
        builder: WebViewBuilder<impl request::RequestHandler>,
        event_handler: impl EventHandler,
        window: &mut Window,
    ) -> Self {
        let WebViewBuilder { init_scripts, request_handler, .. } = builder;

        let view = window.ns_view() as *mut Object;

        unsafe {
            let delegate = WinitDelegate::new(event_handler, request_handler);

            let _: () = msg_send![view, setAutoresizesSubviews: objc::runtime::YES];

            let config: *mut Object = msg_send![class!(WKWebViewConfiguration), new];

            let winit_scheme = NSString::from_str("winit");
            let _: () = msg_send![config, setURLSchemeHandler: delegate forURLScheme: winit_scheme];

            // Enable developer tools if requested
            if builder.platform.enable_debug {
                let preferences: *mut Object = msg_send![config, preferences];
                let number: *mut Object =
                    msg_send![class!(NSNumber), numberWithBool: objc::runtime::YES];
                let key = NSString::from_str("developerExtrasEnabled");
                let _: () = msg_send![preferences, setValue: number forKey: key];
            }

            let manager: *mut Object = msg_send![config, userContentController];

            // Register a custom message handler
            let handler_name = NSString::from_str("WinitMessageHandler");
            let _: () = msg_send![manager, addScriptMessageHandler: delegate name: handler_name];

            // Register all init scripts
            for script in init_scripts {
                let script = NSString::from_str(script.as_str());
                let wk_script: *mut Object = msg_send![class!(WKUserScript), alloc];
                let wk_script: *mut Object = msg_send![wk_script, initWithSource: script injectionTime: 0 forMainFrameOnly: objc::runtime::NO];

                let _: () = msg_send![manager, addUserScript: wk_script];
            }

            let web_view: *mut Object = msg_send![class!(WKWebView), alloc];
            let _: () = msg_send![web_view, setAutoresizingMask: 2u64 | 16u64];

            let bounds: CGRect = msg_send![view, bounds];
            let web_view: *mut Object =
                msg_send![web_view, initWithFrame: bounds configuration: config];
            let _: () = msg_send![web_view, setNavigationDelegate: delegate];

            let _: () = msg_send![view, addSubview: web_view];

            PlatformWebView {
                _delegate: StrongPtr::new(delegate),
                web_view: StrongPtr::new(web_view),
            }
        }
    }

    pub fn navigate(&mut self, target: NavigationTarget) {
        match target {
            NavigationTarget::Url(url) => unsafe {
                let url = NSString::from_str(url);
                let url: *mut Object = msg_send![class!(NSURL), URLWithString: url];
                let request: *mut Object = msg_send![class!(NSURLRequest), alloc];
                let request: *mut Object = msg_send![request, initWithURL: url];

                let _: *mut Object = msg_send![*self.web_view, loadRequest: request];
            },
            NavigationTarget::Html(html) => unsafe {
                let html = NSString::from_str(html);
                let base_url = NSString::from_str("winit://");
                let base_url: *mut Object = msg_send![class!(NSURL), URLWithString: base_url];

                let _: *mut Object = msg_send![*self.web_view, loadHTMLString: html baseURL: base_url];
            }
        }
    }

    pub fn execute(&mut self, js: impl AsRef<str>) {
        fn do_nothing_handler(_result: *mut Object, _error: *mut Object) {}

        unsafe {
            let js = NSString::from_str(js.as_ref());
            let block = ConcreteBlock::new(do_nothing_handler);
            let block = block.copy();

            let _: () = msg_send![*self.web_view, evaluateJavaScript: js completionHandler: block];
        }
    }

    pub fn title(&self) -> Option<String> {
        unsafe {
            let title: *const NSString = msg_send![*self.web_view, title];
            match title.as_ref() {
                Some(title) => {
                    if title.len() == 0 {
                        None
                    } else {
                        Some(title.as_str().to_owned())
                    }
                }
                None => None
            }
        }
    }
}

def_class! {
    class WinitDelegate<T: EventHandler, K: request::RequestHandler>: NSObject, WKNavigationDelegate {
        ivar event_handler: *mut c_void;
        ivar request_handler: *mut c_void;

        fn initWithHandler(this, event_handler: *mut c_void, requestHandler request_handler: *mut c_void) -> *mut Object {
            unsafe {
                this.set_ivar("event_handler", event_handler);
                this.set_ivar("request_handler", request_handler);

                msg_send![this, init]
            }
        }

        fn dealloc(this) {
            unsafe {
                // Get a pointer to the internal state
                let event_handler = *this.get_ivar::<*mut c_void>("event_handler");
                // Drop the internal state
                Box::from_raw(event_handler as *mut T);

                let _: () = msg_send![super(this, class!(NSObject)), dealloc];
            }
        }

        fn userContentController(this, _user_content_controller: *mut Object, didReceiveScriptMessage message: *mut Object) {
            unsafe {
                let event_handler: *mut c_void = *this.get_ivar("event_handler");
                let event_handler = &mut *(event_handler as *mut T);

                let body: *mut Object = msg_send![message, body];
                let is_str: objc::runtime::BOOL = msg_send![body, isKindOfClass: class!(NSString)];
                if is_str == objc::runtime::YES {
                    event_handler.handle_event(Event::Message((*(body as *mut NSString)).as_str().to_owned()));
                }
            }
        }

        fn webView(this, _web_view: *mut Object, didStartProvisionalNavigation _navigation: *mut Object) {
            let event_handler = unsafe {
                let event_handler: *mut c_void = *this.get_ivar("event_handler");
                &mut *(event_handler as *mut T)
            };

            event_handler.handle_event(Event::Navigation(NavigationEvent::Start));
        }

        fn webView(this, _web_view: *mut Object, didCommitNavigation _navigation: *mut Object) {
            let event_handler = unsafe {
                let event_handler: *mut c_void = *this.get_ivar("event_handler");
                &mut *(event_handler as *mut T)
            };

            event_handler.handle_event(Event::Navigation(NavigationEvent::Commit));
        }

        fn webView(this, _web_view: *mut Object, didFinishNavigation _navigation: *mut Object) {
            let event_handler = unsafe {
                let event_handler: *mut c_void = *this.get_ivar("event_handler");
                &mut *(event_handler as *mut T)
            };

            event_handler.handle_event(Event::Navigation(NavigationEvent::Finish));
        }

        fn webView(this, _web_view: *mut Object, startURLSchemeTask task: *mut Object) {
            use std::io::Read;

            unsafe {
                let request: *mut Object = msg_send![task, request];
                let url: *mut Object = msg_send![request, URL];
                let path: &NSString = msg_send![url, path];
                let url: &NSString = msg_send![url, absoluteString];

                let request_handler: *mut c_void = *this.get_ivar("request_handler");
                let request_handler = &mut *(request_handler as *mut K);

                match request_handler.handle_platform_request(PlatformRequest { uri: path.as_str() }) {
                    Some(request::Response { mut body, mime_type }) => {
                        let mut buffer = Vec::new();
                        let content_len = body.read_to_end(&mut buffer).expect("TODO: Return 503");
                        let data = NSData::from_vec(buffer);

                        let url: *mut Object = msg_send![class!(NSURL), URLWithString: url];
                        let mime_type = NSString::from_str(mime_type.as_str());
                        let response: *mut Object = msg_send![class!(NSURLResponse), alloc];

                        let response: *mut Object = msg_send![response, initWithURL: url MIMEType: mime_type expectedContentLength: content_len textEncodingName: std::ptr::null_mut::<Object>()];
                        let _: () = msg_send![task, didReceiveResponse: response];

                        let _: () = msg_send![task, didReceiveData: data];
                        let _: () = msg_send![task, didFinish];
                    }
                    None => {
                        // IDK lol
                    }
                }
            }
        }
    }
}

impl<T: EventHandler, K: request::RequestHandler> WinitDelegate<T, K> {
    pub fn new(event_handler: T, request_handler: K) -> *mut Object {
        let event_handler = Box::new(event_handler);
        let request_handler = Box::new(request_handler);

        unsafe {
            let del: *mut Object = msg_send![Self::class(), alloc];
            let del: *mut Object = msg_send![del, initWithHandler: Box::into_raw(event_handler) as *mut c_void requestHandler: Box::into_raw(request_handler) as *mut c_void];

            del
        }
    }
}
