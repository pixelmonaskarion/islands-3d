use winit::event_loop::EventLoop;
use runner::common_main;

mod game;
mod water;
mod height_map;
mod runner;
mod instance_compute;
mod banana_instance;
include!(concat!(env!("OUT_DIR"), "/resources.rs"));

#[tokio::main]
async fn main() {
    env_logger::init();
    let event_loop = EventLoop::new().unwrap();
    common_main(event_loop).await;
}