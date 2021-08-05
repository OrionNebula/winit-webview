extern crate winit;
extern crate winit_webview;

use std::task::Waker;

use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    platform::run_return::EventLoopExtRunReturn,
    window::WindowBuilder,
};
use winit_webview as webview;

pub fn main() {
    let mut event_loop = EventLoop::with_user_event();

    let window = WindowBuilder::new()
        .with_inner_size(winit::dpi::LogicalSize::new(640.0, 480.0))
        .build(&event_loop)
        .unwrap();

    let mut web_view = webview::WebViewBuilder::new().build(event_loop.create_proxy(), window);
    web_view.navigate(webview::NavigationTarget::Url("https://i.am.gay"));

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
