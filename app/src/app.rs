use lemon_sand_core::cell::Cell;
use lemon_sand_core::sandbox::Sandbox;
use pixels::{Pixels, SurfaceTexture};
use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::dpi::{LogicalSize, PhysicalPosition};
use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};

#[derive(Debug, Default, Clone, Copy)]
pub enum PlaceMode {
    #[default]
    Sand,
    Water,
}

pub struct App {
    window: Option<Arc<Window>>,
    pixels: Option<Pixels<'static>>,
    pub sandbox: Sandbox,
    paused: bool,
    cursor_pos: PhysicalPosition<f64>,
    cursor_pressed: bool,
    place_mode: PlaceMode,
}

impl App {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            window: None,
            pixels: None,
            sandbox: Sandbox::new(width, height),
            paused: false,
            cursor_pos: PhysicalPosition::default(),
            cursor_pressed: false,
            place_mode: PlaceMode::default(),
        }
    }

    fn place(&mut self, x: isize, y: isize) {
        match self.place_mode {
            PlaceMode::Sand => self.sandbox.set(x, y, Cell::sand()),
            PlaceMode::Water => self.sandbox.set(x, y, Cell::water()),
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attrs = Window::default_attributes()
            .with_title("Lemon Sand")
            .with_inner_size(LogicalSize::new(
                self.sandbox.width() as f64 * 5.0,
                self.sandbox.height() as f64 * 5.0,
            ))
            .with_min_inner_size(LogicalSize::new(
                self.sandbox.width() as f64,
                self.sandbox.height() as f64,
            ))
            .with_resize_increments(LogicalSize::new(
                self.sandbox.width() as f64,
                self.sandbox.height() as f64,
            ));

        let window = Arc::new(event_loop.create_window(window_attrs).unwrap());

        let size = window.inner_size();
        let surface = SurfaceTexture::new(size.width, size.height, window.clone());
        let mut pixels = Pixels::new(
            self.sandbox.width() as u32,
            self.sandbox.height() as u32,
            surface,
        )
        .unwrap();

        pixels.clear_color(pixels::wgpu::Color {
            r: 0.1,
            g: 0.1,
            b: 0.1,
            a: 1.0,
        });

        self.window = Some(window);
        self.pixels = Some(pixels);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::RedrawRequested => {
                if let Some(pixels) = &mut self.pixels {
                    self.sandbox.draw(pixels.frame_mut());
                    pixels.render().unwrap();
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                let PhysicalKey::Code(code) = event.physical_key else {
                    return;
                };

                match code {
                    KeyCode::Digit1 => self.place_mode = PlaceMode::Sand,
                    KeyCode::Digit2 => self.place_mode = PlaceMode::Water,
                    KeyCode::Space => self.paused = !self.paused,
                    _ => {}
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.cursor_pos = position;
            }
            WindowEvent::MouseInput { state, button, .. } => {
                if button == MouseButton::Left {
                    match state {
                        ElementState::Pressed => self.cursor_pressed = true,
                        ElementState::Released => self.cursor_pressed = false,
                    }
                }
            }
            WindowEvent::Resized(size) => {
                if let Some(pixels) = &mut self.pixels
                    && size.width > 0
                    && size.height > 0
                {
                    pixels.resize_surface(size.width, size.height).unwrap();
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if self.cursor_pressed
            && let Some(pixels) = &self.pixels
            && let Ok((x, y)) =
                pixels.window_pos_to_pixel((self.cursor_pos.x as f32, self.cursor_pos.y as f32))
        {
            self.place(x as isize, y as isize);
        }

        if !self.paused {
            self.sandbox.update();
        }

        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}
