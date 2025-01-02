use std::time::{Duration, Instant};

use cgmath::{Quaternion, Rotation3, SquareMatrix, Vector3};
use wgpu::Color;
use winit::{
    event::{DeviceEvent, ElementState, Event, KeyEvent, WindowEvent},
    event_loop::EventLoop,
    keyboard::{Key, KeyCode, NamedKey, PhysicalKey},
};

use crate::{
    camera::{self, Camera, CameraController, CameraUniform, Projection},
    chunk::{generate_chunks, Chunk, ChunkList},
    renderer::{self, block::Block, renderer::Renderer},
};

struct KeyEntry(KeyCode, ElementState);

pub struct Game<'a> {
    // The window of the game.
    window: &'a winit::window::Window,

    // The game renderer.
    renderer: Renderer<'a>,

    /// The time in seconds since the last frame.
    delta: f32,
    /// The key events that have been received since the last frame.
    key_events: Vec<KeyEntry>,
    /// Whether the game should close.
    should_close: bool,

    camera_controller: CameraController,
    camera: Camera,

    chunk_list: ChunkList,
}

impl<'a> Game<'a> {
    pub fn new(window: &'a winit::window::Window, renderer: Renderer<'a>) -> Self {
        let size = window.inner_size();
        let projection =
            camera::Projection::new(size.width, size.height, cgmath::Deg(45.0), 0.5, 100.0);
        let camera = camera::Camera::new(
            (0.0, 5.0, 10.0),
            cgmath::Deg(-90.0),
            cgmath::Deg(-20.0),
            projection,
        );
        Self {
            window,
            renderer,
            delta: 0.0,
            key_events: Vec::new(),
            should_close: false,
            camera_controller: CameraController::new(10.0, 4.0),
            camera,
            chunk_list: ChunkList::new(generate_chunks(16)),
        }
    }

    fn update(&mut self) {
        self.key_events.iter().for_each(|KeyEntry(key, state)| {
            if *state == ElementState::Pressed && *key == KeyCode::Escape {
                self.should_close = true
            } else {
                self.camera_controller.process_keyboard(*key, *state);
            }
        });

        self.key_events.clear();
        self.camera_controller
            .update_camera(&mut self.camera, self.delta);

        let camera_uniform = CameraUniform::init(&self.camera);
        self.renderer.update_camera_uniform(camera_uniform);
    }

    fn render(&mut self) {
        let mesh = self.chunk_list.mesh();
        self.renderer.draw_terrain(&mesh);
    }

    pub async fn run(&mut self, event_loop: EventLoop<()>) {
        let window_id = self.window.id();
        let mut surface_configured = false;
        let mut last_frame_time = Instant::now();

        event_loop
            .run(move |event, control_flow| {
                if self.should_close {
                    control_flow.exit();
                }

                match event {
                    Event::DeviceEvent {
                        event: DeviceEvent::MouseMotion { delta },
                        ..
                    } => {
                        self.camera_controller.process_mouse(delta.0, delta.1);
                    }
                    Event::WindowEvent {
                        ref event,
                        window_id: event_window_id,
                    } if event_window_id == window_id => match event {
                        WindowEvent::Resized(physical_size) => {
                            self.renderer.on_resize(*physical_size);
                            self.camera
                                .projection
                                .resize(physical_size.width, physical_size.height);
                            surface_configured = true;
                        }
                        WindowEvent::CloseRequested => control_flow.exit(),
                        WindowEvent::KeyboardInput {
                            event:
                                KeyEvent {
                                    physical_key: PhysicalKey::Code(key),
                                    state,
                                    ..
                                },
                            ..
                        } => self.key_events.push(KeyEntry(*key, *state)),
                        WindowEvent::RedrawRequested => {
                            self.window.request_redraw();

                            if !surface_configured {
                                return;
                            }

                            let now = Instant::now();
                            self.delta = (now - last_frame_time).as_secs_f32();
                            last_frame_time = now;

                            println!("FPS: {}", 1.0 / self.delta);

                            self.update();
                            self.render();
                        }
                        _ => {}
                    },
                    _ => {}
                }
            })
            .unwrap();
    }
}
