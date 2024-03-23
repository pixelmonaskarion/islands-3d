use game::Game;
use winit::{event_loop::EventLoop, window::WindowBuilder};
use bespoke_engine::window::Surface;

mod game;
mod water;
mod height_map;
mod instance;

#[tokio::main]
async fn main() {
    env_logger::init();
    let event_loop = EventLoop::new().unwrap();
    common_main(event_loop).await;
}

async fn common_main<T>(event_loop: EventLoop<T>) {
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    let surface = Surface::new(&window).await;
    window.set_cursor_grab(winit::window::CursorGrabMode::Locked).unwrap();
    window.set_cursor_visible(false);
    let game = Game::new(&surface.device, &surface.queue, surface.config.format, window.inner_size());
    surface.run(game, event_loop);
}