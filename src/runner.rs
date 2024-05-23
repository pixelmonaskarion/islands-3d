use bespoke_engine::window::{Surface, SurfaceContext};
use winit::event_loop::EventLoop;

use crate::game::Game;

#[allow(dead_code)]
pub async fn common_main(event_loop: EventLoop<()>) {
    let ready = &|surface_context: &SurfaceContext| {
        let _ = surface_context.window.set_cursor_grab(winit::window::CursorGrabMode::Locked);
        Game::new(&surface_context.device, &surface_context.queue, surface_context.config.format, surface_context.window.inner_size())
    };
    let mut surface = Surface::new(ready).await;
    event_loop.run_app(&mut surface).unwrap();
}