use bespoke_engine::window::{Surface, SurfaceContext};
use winit::{event_loop::EventLoop, window::WindowBuilder};

use crate::game::Game;

#[allow(dead_code)]
pub async fn common_main<T>(event_loop: EventLoop<T>) {
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    let surface = Surface::new(&window).await;
    let _ = window.set_cursor_grab(winit::window::CursorGrabMode::Locked);
    window.set_cursor_visible(false);
    surface.run(event_loop, &|surface_context: &SurfaceContext| {
        Game::new(&surface_context.device, &surface_context.queue, surface_context.config.format, window.inner_size())
    });
}