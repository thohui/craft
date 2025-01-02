#![allow(warnings)]
use game::Game;
use winit::{event_loop::EventLoop, window::Window};

mod camera;
mod chunk;
mod game;
mod noise;
mod renderer;

#[tokio::main]
async fn main() {
    let event_loop = EventLoop::new().unwrap();
    let window = Window::new(&event_loop).unwrap();
    let renderer = renderer::renderer::Renderer::new(&window).await;

    let mut game = Game::new(&window, renderer);
    game.run(event_loop).await;
}
