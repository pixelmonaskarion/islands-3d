use game::Game;
use winit::{event_loop::EventLoop, window::WindowBuilder};
use bespoke_engine::window::{Surface, SurfaceContext};

#[cfg(target_os = "android")] 
use winit::platform::android::activity::AndroidApp;
#[cfg(target_os = "android")] 
use winit::event_loop::EventLoopBuilder;

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

#[allow(dead_code)]
#[cfg(target_os = "android")]
#[no_mangle]
fn android_main(app: AndroidApp) {
    use winit::platform::android::EventLoopBuilderExtAndroid;

    android_logger::init_once(android_logger::Config::default().with_max_level(log::LevelFilter::Info));

    let event_loop = EventLoopBuilder::new().with_android_app(app).build().unwrap();
    pollster::block_on(common_main(event_loop));
}

async fn common_main<T>(event_loop: EventLoop<T>) {
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    let surface = Surface::new(&window).await;
    let _ = window.set_cursor_grab(winit::window::CursorGrabMode::Locked);
    window.set_cursor_visible(false);
    surface.run(event_loop, &|surface_context: &SurfaceContext| {
        Game::new(&surface_context.device, &surface_context.queue, surface_context.config.format, window.inner_size())
    });
}