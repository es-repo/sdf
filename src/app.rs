use crate::AppEvent;
use crate::fps_counter::FpsCounter;
use font8x8::UnicodeFonts;
use pixels::Pixels;
use sdf::scenes::Scene;
use sdf::{ColorExt, Vec2};
use std::sync::Arc;
use web_time::Instant;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoopProxy};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowAttributes, WindowId};

#[cfg(target_arch = "wasm32")]
use winit::platform::web::WindowAttributesExtWebSys;
#[cfg(not(target_arch = "wasm32"))]
use winit::dpi::PhysicalPosition;

pub struct App {
    window: Option<Arc<Window>>,
    pixels: Option<Pixels<'static>>,
    scene: Box<dyn Scene>,
    event_proxy: EventLoopProxy<AppEvent>,
    size_logical: LogicalSize<u32>,
    scale_factor: f64,
    start_time: Instant,
    fps_counter: FpsCounter,
    show_fps: bool,
}

impl App {
    pub fn new(
        size_logical: LogicalSize<u32>,
        pixel_color_source: impl Scene + 'static,
        event_proxy: EventLoopProxy<AppEvent>,
    ) -> Self {
        Self {
            window: None,
            pixels: None,
            scene: Box::new(pixel_color_source),
            event_proxy,
            size_logical,
            start_time: Instant::now(),
            fps_counter: FpsCounter::new(),
            show_fps: false,
            scale_factor: 1.0,
        }
    }
}

impl ApplicationHandler<AppEvent> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.pixels.is_none() {
            let mut window_attributes = WindowAttributes::default()
                .with_title("SDF")
                .with_inner_size(self.size_logical);

            #[cfg(target_arch = "wasm32")]
            {
                window_attributes = window_attributes.with_append(true).with_focusable(true);
            }

            let window = Arc::new(event_loop.create_window(window_attributes).unwrap());

            #[cfg(not(target_arch = "wasm32"))]
            if let Some(monitor) = event_loop.available_monitors().nth(2) {
                let monitor_pos = monitor.position();
                let monitor_size = monitor.size();
                let window_size = window.outer_size();

                let x = monitor_pos.x + (monitor_size.width as i32 - window_size.width as i32) / 2;
                let y = monitor_pos.y + (monitor_size.height as i32 - window_size.height as i32) / 2;

                window.set_outer_position(PhysicalPosition::new(x, y));
            }

            window.request_redraw();

            self.scale_factor = window.scale_factor();
            let size_physical = self.size_logical.to_physical(self.scale_factor);
            self.window = Some(window);
            self.start_time = Instant::now();
            self.fps_counter.reset();

            #[cfg(not(target_arch = "wasm32"))]
            {
                let surface_texture = pixels::SurfaceTexture::new(
                    size_physical.width,
                    size_physical.height,
                    Arc::clone(self.window.as_ref().unwrap()),
                );

                let pixels = pixels::PixelsBuilder::new(
                    self.size_logical.width,
                    self.size_logical.height,
                    surface_texture,
                )
                .enable_vsync(true)
                .build()
                .unwrap();

                self.pixels = Some(pixels);
            }

            #[cfg(target_arch = "wasm32")]
            {
                let surface_texture = pixels::SurfaceTexture::new(
                    size_physical.width,
                    size_physical.height,
                    Arc::clone(self.window.as_ref().unwrap()),
                );
                let proxy = self.event_proxy.clone();
                let width = self.size_logical.width;
                let height = self.size_logical.height;
                let device_descriptor = pixels::wgpu::DeviceDescriptor {
                    label: Some("sdf-web-device"),
                    required_features: pixels::wgpu::Features::empty(),
                    required_limits: pixels::wgpu::Limits::downlevel_webgl2_defaults(),
                };

                wasm_bindgen_futures::spawn_local(async move {
                    let pixels = pixels::PixelsBuilder::new(width, height, surface_texture)
                        .wgpu_backend(pixels::wgpu::Backends::GL)
                        .device_descriptor(device_descriptor)
                        .enable_vsync(true)
                        .build_async()
                        .await
                        .unwrap();

                    proxy.send_event(AppEvent::PixelsReady(pixels)).unwrap();
                });
            }
        }
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: AppEvent) {
        match event {
            AppEvent::PixelsReady(pixels) => {
                self.pixels = Some(pixels);
                self.window.as_ref().unwrap().request_redraw();
            }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),

            WindowEvent::RedrawRequested => {
                let Some(pixels) = self.pixels.as_mut() else {
                    return;
                };

                let frame = pixels.frame_mut();

                let elapsed = self.start_time.elapsed();
                let time = elapsed.as_secs_f64();
                self.fps_counter.tick();

                for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
                    let x = i as u32 % self.size_logical.width;
                    let y = i as u32 / self.size_logical.width;
                    let coord = Vec2::new(x, y).to_aspect_ndc(self.size_logical.width, self.size_logical.height);

                    let color = self.scene.get_pixel_color(coord, time);
                    pixel.copy_from_slice(&color.to_u8_array());
                }

                if self.show_fps {
                    let fps_text = format!("{:.0}", self.fps_counter.count());
                    draw_text(
                        frame,
                        self.size_logical.width,
                        self.size_logical.height,
                        &fps_text,
                        16,
                        16,
                        4,
                        [255, 255, 255, 255],
                    );
                }

                pixels.render().unwrap();
                self.window.as_ref().unwrap().request_redraw();
            }

            WindowEvent::KeyboardInput { event, .. } => {
                if event.state == ElementState::Pressed
                    && !event.repeat
                    && matches!(event.physical_key, PhysicalKey::Code(KeyCode::KeyF))
                {
                    self.show_fps = !self.show_fps;
                    self.window.as_ref().unwrap().request_redraw();
                }
            }

            WindowEvent::Resized(size_physical) => {
                if size_physical.width > 0 && size_physical.height > 0 {
                    let Some(pixels) = self.pixels.as_mut() else {
                        self.size_logical = size_physical.to_logical::<u32>(self.scale_factor);
                        return;
                    };

                    pixels
                        .resize_surface(size_physical.width, size_physical.height)
                        .unwrap();

                    self.size_logical = size_physical.to_logical::<u32>(self.scale_factor);
                    pixels
                        .resize_buffer(self.size_logical.width, self.size_logical.height)
                        .unwrap();

                    self.window.as_ref().unwrap().request_redraw();
                }
            }

            WindowEvent::ScaleFactorChanged {
                scale_factor,
                inner_size_writer: _inner_size_writer,
            } => {
                self.scale_factor = scale_factor;
                self.window.as_ref().unwrap().request_redraw();
            }

            _ => {}
        }
    }
}

fn draw_text(frame: &mut [u8], width: u32, height: u32, text: &str, x: i32, y: i32, scale: i32, color: [u8; 4]) {
    let mut pen_x = x;

    for ch in text.chars() {
        if let Some(glyph) = font8x8::BASIC_FONTS.get(ch) {
            for (row, bits) in glyph.iter().enumerate() {
                for col in 0..8 {
                    if ((bits >> col) & 1) == 0 {
                        continue;
                    }

                    for sy in 0..scale {
                        for sx in 0..scale {
                            let px = pen_x + col * scale + sx;
                            let py = y + row as i32 * scale + sy;

                            if px < 0 || py < 0 || px >= width as i32 || py >= height as i32 {
                                continue;
                            }

                            let index = ((py as u32 * width + px as u32) * 4) as usize;
                            frame[index..index + 4].copy_from_slice(&color);
                        }
                    }
                }
            }
        }

        pen_x += 8 * scale + scale;
    }
}
