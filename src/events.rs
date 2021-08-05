use std::sync::mpsc;

pub use crate::platform_impl::PlatformEvent;
pub use crate::platform_impl::PlatformNavigationEvent;

/// An event emitted by the WebView component.
#[derive(Debug, Clone)]
pub enum Event {
    /// Navigation status has changed.
    Navigation(NavigationEvent),
    /// A script has sent a message to the host webview.
    Message(String),
    /// A platform-specific event has occurred.
    Platform(PlatformEvent)
}

/// Navigation status has changed.
#[derive(Debug, Clone)]
pub enum NavigationEvent {
    /// Triggered when navigation is initiated
    Start,
    /// Triggered when navigation recieves content and begins loading it
    Commit,
    /// Triggered when navigation is complete
    Finish,

    /// Some other platform-specific navigation event
    Platform(PlatformNavigationEvent),
}

/// A recipient for WebView events.
pub trait EventHandler: 'static + Sized {
    fn handle_event(&mut self, event: Event);
}

impl<T: From<Event>> EventHandler for winit::event_loop::EventLoopProxy<T> {
    fn handle_event(&mut self, event: Event) {
        winit::event_loop::EventLoopProxy::<T>::send_event(self, T::from(event)).ok();
    }
}


impl<T> EventHandler for T where T: 'static + FnMut(Event) {
    fn handle_event(&mut self, event: Event) {
        (self)(event)
    }
}

impl EventHandler for mpsc::Sender<Event> {
    fn handle_event(&mut self, event: Event) {
        self.send(event).ok();
    }
}
