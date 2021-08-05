extern crate mime_guess;
extern crate winit;
extern crate winit_webview;

use std::path::Path;

use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    platform::run_return::EventLoopExtRunReturn,
    window::WindowBuilder,
};
use winit_webview as webview;

struct RequestHandler;

impl webview::request::RequestHandler for RequestHandler {
    type Read = std::fs::File;

    fn handle_request(&mut self, uri: &str) -> Option<webview::request::Response<Self::Read>> {
        let path = Path::new("./examples/dist").join(Path::new(uri).strip_prefix("/").ok()?);

        let file = std::fs::File::open(&path).ok()?;
        if file.metadata().ok()?.is_file() {
            Some(webview::request::Response {
                body: file,
                mime_type: mime_guess::from_ext(path.extension()?.to_str()?)
                    .first_raw()?
                    .to_owned(),
            })
        } else {
            None
        }
    }
}

pub fn main() {
    use webview::platform::macos::WebViewBuilderExtMacOS;

    let mut event_loop = EventLoop::with_user_event();

    let window = WindowBuilder::new()
        .with_inner_size(winit::dpi::LogicalSize::new(640.0, 480.0))
        .build(&event_loop)
        .unwrap();

    let mut web_view = webview::WebViewBuilder::with_request_handler(RequestHandler)
        .with_debug(true)
        .build(event_loop.create_proxy(), window);
    web_view.navigate(webview::NavigationTarget::Url("winit:///index.html"));

    event_loop.run_return(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
            } if window_id == web_view.window.id() => *control_flow = ControlFlow::Exit,
            Event::UserEvent(webview::Event::Navigation(webview::NavigationEvent::Finish)) => {
                if let Some(title) = web_view.title() {
                    web_view.window.set_title(&title);
                }
            }
            Event::UserEvent(evt) => {
                println!("{:?}", evt);
            }
            _ => (),
        }
    });
}
