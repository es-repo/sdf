use crate::fps_counter::FpsCounter;
use font8x8::UnicodeFonts;
use pixels::{Pixels, ScalingMode};
use rayon::prelude::*;
use sdf::audio::{AudioAnalysis, AudioTrack};
use sdf::color_ext::ColorExt;
use sdf::geometry::Vec2;
use sdf::input::InputState;
use sdf::scenes::SceneInstance;
use std::sync::Arc;
use web_time::Instant;
use winit::application::ApplicationHandler;
use winit::dpi::{LogicalSize, PhysicalSize};
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowAttributes, WindowId};

#[cfg(target_arch = "wasm32")]
use crate::wasm_boot::AppEvent;
#[cfg(target_os = "macos")]
use core_graphics::display::CGDisplay;
#[cfg(target_os = "macos")]
use core_graphics::event::CGEvent;
#[cfg(target_os = "macos")]
use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};
use egui_wgpu::{Renderer, RendererOptions, ScreenDescriptor};
use egui_winit::State as EguiWinitState;
#[cfg(not(target_arch = "wasm32"))]
use winit::dpi::PhysicalPosition;
#[cfg(target_arch = "wasm32")]
use winit::event_loop::EventLoopProxy;
#[cfg(target_os = "macos")]
use winit::platform::macos::MonitorHandleExtMacOS;
#[cfg(target_arch = "wasm32")]
use winit::platform::web::WindowAttributesExtWebSys;

pub struct Viewer {
    window: Option<Arc<Window>>,
    pixels: Option<Pixels<'static>>,
    scene: SceneInstance,
    egui: Option<EguiState>,
    #[cfg(target_arch = "wasm32")]
    event_proxy: EventLoopProxy<AppEvent>,
    size_logical: LogicalSize<u32>,
    scale_factor: f64,
    start_time: Instant,
    last_frame_time: Instant,
    input: InputState,
    fps_counter: FpsCounter,
    show_fps: bool,
    audio_track: Option<AudioTrack>,
    #[cfg(target_arch = "wasm32")]
    audio_enabled: bool,
}

struct EguiState {
    context: egui::Context,
    state: EguiWinitState,
    renderer: Renderer,
}

struct EguiFrame {
    paint_jobs: Vec<egui::ClippedPrimitive>,
    screen_descriptor: ScreenDescriptor,
    textures_delta: egui::TexturesDelta,
}

impl EguiState {
    fn new(window: &Window, pixels: &Pixels<'static>) -> Self {
        let context = egui::Context::default();
        let state = EguiWinitState::new(
            context.clone(),
            egui::ViewportId::ROOT,
            window,
            Some(window.scale_factor() as f32),
            None,
            None,
        );
        let renderer = Renderer::new(
            &pixels.context().device,
            pixels.surface_texture_format(),
            RendererOptions::default(),
        );

        Self {
            context,
            state,
            renderer,
        }
    }
}

impl Viewer {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn new(size_logical: LogicalSize<u32>, scene: SceneInstance) -> Self {
        let now = Instant::now();

        Self {
            window: None,
            pixels: None,
            scene,
            egui: None,
            size_logical,
            start_time: now,
            last_frame_time: now,
            input: InputState::default(),
            fps_counter: FpsCounter::new(),
            show_fps: false,
            scale_factor: 1.0,
            audio_track: None,
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub fn new(size_logical: LogicalSize<u32>, scene: SceneInstance, event_proxy: EventLoopProxy<AppEvent>) -> Self {
        let now = Instant::now();

        Self {
            window: None,
            pixels: None,
            scene,
            egui: None,
            event_proxy,
            size_logical,
            start_time: now,
            last_frame_time: now,
            input: InputState::default(),
            fps_counter: FpsCounter::new(),
            show_fps: false,
            scale_factor: 1.0,
            audio_track: None,
            audio_enabled: false,
        }
    }

    fn sync_scene_audio(&mut self) {
        let track = self.scene.audio_track();
        let volume = self.scene.audio_volume();

        if !self.is_audio_enabled() {
            if let Some(audio_track) = self.audio_track.as_mut() {
                audio_track.play(None);
            }

            return;
        }

        if track.is_some() && self.audio_track.is_none() {
            self.audio_track = AudioTrack::new(audio_base_path());
        }

        if let Some(audio_track) = self.audio_track.as_mut() {
            audio_track.play(track);
            audio_track.set_volume(volume);
        }
    }

    fn sync_audio_volume(&mut self) {
        let volume = self.scene.audio_volume();

        if let Some(audio_track) = self.audio_track.as_mut() {
            audio_track.set_volume(volume);
        }
    }

    fn audio_analysis(&mut self) -> AudioAnalysis {
        self.audio_track.as_mut().map(AudioTrack::analysis).unwrap_or_default()
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn is_audio_enabled(&self) -> bool {
        true
    }

    #[cfg(target_arch = "wasm32")]
    fn is_audio_enabled(&self) -> bool {
        self.audio_enabled
    }

    #[cfg(target_arch = "wasm32")]
    fn set_audio_enabled(&mut self, enabled: bool) {
        self.audio_enabled = enabled;
        self.sync_scene_audio();

        if let Some(window) = self.window.as_ref() {
            window.request_redraw();
        }
    }

    fn prepare_egui_frame(&mut self) -> Option<EguiFrame> {
        let scene = self.scene.parameterized_scene_mut()?;
        let window = self.window.as_ref()?;
        let egui = self.egui.as_mut()?;
        let mut raw_input = egui.state.take_egui_input(window);

        // Browser device-pixel sizes can truncate fractional CSS pixels while
        // LogicalSize::to_physical can round them up. Keep egui's scissor
        // rectangles no larger than the actual render target.
        let expected_surface_size = self.size_logical.to_physical::<u32>(self.scale_factor);
        let window_surface_size = window.inner_size();
        let surface_size = PhysicalSize::new(
            expected_surface_size.width.min(window_surface_size.width),
            expected_surface_size.height.min(window_surface_size.height),
        );
        let pixels_per_point = egui_winit::pixels_per_point(&egui.context, window);
        let screen_size_in_points = egui::vec2(
            surface_size.width as f32 / pixels_per_point,
            surface_size.height as f32 / pixels_per_point,
        );
        let screen_rect = egui::Rect::from_min_size(egui::Pos2::ZERO, screen_size_in_points);
        raw_input.screen_rect = Some(screen_rect);
        raw_input.viewports.entry(raw_input.viewport_id).or_default().inner_rect = Some(screen_rect);

        let full_output = egui.context.run(raw_input, |context| {
            let original_style = context.style();
            let mut compact_style = (*original_style).clone();
            compact_style
                .text_styles
                .insert(egui::TextStyle::Heading, egui::FontId::proportional(13.0));
            compact_style.spacing.icon_width = 12.0;
            compact_style.spacing.interact_size.y = 16.0;
            compact_style.spacing.window_margin = egui::Margin::symmetric(6, 2);
            let compact_frame = egui::Frame::window(&compact_style);
            context.set_style(compact_style);

            egui::Window::new("Parameters")
                .anchor(egui::Align2::RIGHT_TOP, egui::vec2(0.0, 0.0))
                .default_width(220.0)
                .frame(compact_frame)
                .resizable(false)
                .collapsible(true)
                .default_open(false)
                .show(context, |ui| {
                    scene.parameters_ui(ui);
                });

            context.set_style(original_style);
        });

        egui.state.handle_platform_output(window, full_output.platform_output);

        let pixels_per_point = full_output.pixels_per_point;
        let paint_jobs = egui.context.tessellate(full_output.shapes, pixels_per_point);

        Some(EguiFrame {
            paint_jobs,
            screen_descriptor: ScreenDescriptor {
                size_in_pixels: [surface_size.width, surface_size.height],
                pixels_per_point,
            },
            textures_delta: full_output.textures_delta,
        })
    }

    fn render(&mut self) {
        if self.pixels.is_none() {
            return;
        }

        let now = Instant::now();
        let elapsed = now.duration_since(self.start_time);
        let delta_time = now.duration_since(self.last_frame_time).as_secs_f32();
        self.last_frame_time = now;
        let time = elapsed.as_secs_f64() as f32;
        self.fps_counter.tick();
        let egui_frame = self.prepare_egui_frame();
        self.sync_audio_volume();
        let audio_analysis = self.audio_analysis();
        self.scene.update(delta_time, &self.input);
        self.input.reset_frame_delta();
        let prepared_scene = self.scene.prepare_frame_with_audio(time, &audio_analysis);
        let width = self.size_logical.width;
        let height = self.size_logical.height;
        let row_stride = width as usize * 4;
        let height_f = height as f32;
        let width_f = width as f32;
        let dx = 2.0 / height_f;

        let frame = self.pixels.as_mut().unwrap().frame_mut();

        frame.par_chunks_exact_mut(row_stride).enumerate().for_each(|(y, row)| {
            let y = y as f32;
            let ny = (height_f - 2.0 * (y + 0.5)) / height_f;
            let mut nx = (1.0 - width_f) / height_f;

            for pixel in row.chunks_exact_mut(4) {
                let coord = Vec2::new(nx, ny);
                let color = prepared_scene.get_pixel_color_with_audio(coord, time, &audio_analysis);
                pixel.copy_from_slice(&color.to_u8_array());
                nx += dx;
            }
        });

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

        self.render_with_egui(egui_frame);

        self.window.as_ref().unwrap().request_redraw();
    }

    fn render_with_egui(&mut self, egui_frame: Option<EguiFrame>) {
        let Some(egui_frame) = egui_frame else {
            self.pixels.as_ref().unwrap().render().unwrap();
            return;
        };
        let Some(egui) = self.egui.as_mut() else {
            self.pixels.as_ref().unwrap().render().unwrap();
            return;
        };

        self.pixels
            .as_ref()
            .unwrap()
            .render_with(|encoder, render_target, context| {
                context.scaling_renderer.render(encoder, render_target);

                for (texture_id, image_delta) in &egui_frame.textures_delta.set {
                    egui.renderer
                        .update_texture(&context.device, &context.queue, *texture_id, image_delta);
                }

                let command_buffers = egui.renderer.update_buffers(
                    &context.device,
                    &context.queue,
                    encoder,
                    &egui_frame.paint_jobs,
                    &egui_frame.screen_descriptor,
                );

                context.queue.submit(command_buffers);

                {
                    let render_pass = encoder.begin_render_pass(&pixels::wgpu::RenderPassDescriptor {
                        label: Some("egui_render_pass"),
                        color_attachments: &[Some(pixels::wgpu::RenderPassColorAttachment {
                            view: render_target,
                            resolve_target: None,
                            ops: pixels::wgpu::Operations {
                                load: pixels::wgpu::LoadOp::Load,
                                store: pixels::wgpu::StoreOp::Store,
                            },
                            depth_slice: None,
                        })],
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                    });

                    egui.renderer.render(
                        &mut render_pass.forget_lifetime(),
                        &egui_frame.paint_jobs,
                        &egui_frame.screen_descriptor,
                    );
                }

                for texture_id in &egui_frame.textures_delta.free {
                    egui.renderer.free_texture(texture_id);
                }

                Ok(())
            })
            .unwrap();
    }

    fn handle_window_event(&mut self, event_loop: &ActiveEventLoop, event: WindowEvent) {
        let gui_consumed = self.handle_gui_window_event(&event);

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),

            WindowEvent::RedrawRequested => self.render(),

            WindowEvent::KeyboardInput { event, .. } => {
                if let PhysicalKey::Code(key) = event.physical_key {
                    match event.state {
                        ElementState::Pressed if !gui_consumed => self.input.set_key_pressed(key, true),
                        ElementState::Released => self.input.set_key_pressed(key, false),
                        _ => {}
                    }
                }

                if event.state == ElementState::Pressed && !event.repeat {
                    self.sync_scene_audio();

                    if !gui_consumed && matches!(event.physical_key, PhysicalKey::Code(KeyCode::KeyF)) {
                        self.show_fps = !self.show_fps;
                        self.window.as_ref().unwrap().request_redraw();
                    }
                }
            }

            WindowEvent::MouseInput { state, button, .. } => {
                match state {
                    ElementState::Pressed if !gui_consumed => self.input.set_mouse_button_pressed(button, true),
                    ElementState::Released => self.input.set_mouse_button_pressed(button, false),
                    _ => {}
                }

                if state == ElementState::Pressed {
                    self.sync_scene_audio();
                }
            }

            WindowEvent::CursorMoved { position, .. } => {
                self.input
                    .set_mouse_position(Vec2::new(position.x as f32, position.y as f32));
            }

            WindowEvent::CursorLeft { .. } => self.input.clear_mouse_position(),

            WindowEvent::Focused(false) => self.input.clear(),

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

    fn handle_gui_window_event(&mut self, event: &WindowEvent) -> bool {
        if self.scene.parameterized_scene().is_none() {
            return false;
        }

        let Some(window) = self.window.as_ref() else {
            return false;
        };
        let Some(egui) = self.egui.as_mut() else {
            return false;
        };

        let response = egui.state.on_window_event(window, event);
        if response.repaint {
            window.request_redraw();
        }

        response.consumed
    }

    #[cfg(target_arch = "wasm32")]
    fn resize_scene(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }

        self.size_logical = LogicalSize::new(width, height);

        let Some(window) = self.window.as_ref() else {
            return;
        };

        let _ = window.request_inner_size(self.size_logical);

        if let Some(pixels) = self.pixels.as_mut() {
            let size_physical = self.size_logical.to_physical(self.scale_factor);

            pixels
                .resize_surface(size_physical.width, size_physical.height)
                .unwrap();
            pixels
                .resize_buffer(self.size_logical.width, self.size_logical.height)
                .unwrap();
        }

        window.request_redraw();
    }

    fn prepare_window(&mut self, event_loop: &ActiveEventLoop, window_attributes: WindowAttributes) -> Arc<Window> {
        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());

        #[cfg(target_os = "macos")]
        if let Some(monitor) = monitor_under_cursor(event_loop) {
            let monitor_pos = monitor.position();
            let monitor_size = monitor.size();
            let window_size = window.outer_size();

            let x = monitor_pos.x + (monitor_size.width as i32 - window_size.width as i32) / 2;
            let y = monitor_pos.y + (monitor_size.height as i32 - window_size.height as i32) / 2;

            window.set_outer_position(PhysicalPosition::new(x, y));
        }

        window.request_redraw();

        self.scale_factor = window.scale_factor();
        self.window = Some(Arc::clone(&window));
        self.start_time = Instant::now();
        self.last_frame_time = self.start_time;
        self.fps_counter.reset();

        window
    }

    fn create_surface_texture(&self) -> pixels::SurfaceTexture<Arc<Window>> {
        let window = Arc::clone(self.window.as_ref().unwrap());
        let size_physical = self.size_logical.to_physical(self.scale_factor);

        pixels::SurfaceTexture::new(size_physical.width, size_physical.height, window)
    }

    fn base_window_attributes(&self) -> WindowAttributes {
        WindowAttributes::default()
            .with_title("SDF")
            .with_inner_size(self.size_logical)
    }
}

#[cfg(target_os = "macos")]
fn monitor_under_cursor(event_loop: &ActiveEventLoop) -> Option<winit::monitor::MonitorHandle> {
    let event_source = CGEventSource::new(CGEventSourceStateID::CombinedSessionState).ok()?;
    let cursor = CGEvent::new(event_source).ok()?.location();

    let cursor_display_id = CGDisplay::active_displays().ok()?.into_iter().find(|display_id| {
        let bounds = CGDisplay::new(*display_id).bounds();
        let min_x = bounds.origin.x;
        let max_x = min_x + bounds.size.width;
        let min_y = bounds.origin.y;
        let max_y = min_y + bounds.size.height;

        cursor.x >= min_x && cursor.x < max_x && cursor.y >= min_y && cursor.y < max_y
    })?;

    event_loop
        .available_monitors()
        .find(|monitor| monitor.native_id() == cursor_display_id)
}

#[cfg(not(target_arch = "wasm32"))]
impl ApplicationHandler<()> for Viewer {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.pixels.is_some() {
            return;
        }

        let window = self.prepare_window(event_loop, self.base_window_attributes());
        let surface_texture = self.create_surface_texture();

        let mut pixels = pixels::PixelsBuilder::new(self.size_logical.width, self.size_logical.height, surface_texture)
            .enable_vsync(true)
            .build()
            .unwrap();
        pixels.set_scaling_mode(ScalingMode::Fill);

        self.egui = Some(EguiState::new(&window, &pixels));
        self.pixels = Some(pixels);
        self.sync_scene_audio();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent) {
        self.handle_window_event(event_loop, event);
    }
}

#[cfg(target_arch = "wasm32")]
impl ApplicationHandler<AppEvent> for Viewer {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.pixels.is_some() {
            return;
        }

        let window_attributes = self.base_window_attributes().with_append(true).with_focusable(true);

        self.prepare_window(event_loop, window_attributes);
        let surface_texture = self.create_surface_texture();
        let proxy = self.event_proxy.clone();
        let width = self.size_logical.width;
        let height = self.size_logical.height;
        let device_descriptor = pixels::wgpu::DeviceDescriptor {
            label: Some("sdf-web-device"),
            required_features: pixels::wgpu::Features::empty(),
            required_limits: pixels::wgpu::Limits::downlevel_webgl2_defaults(),
            experimental_features: pixels::wgpu::ExperimentalFeatures::disabled(),
            memory_hints: pixels::wgpu::MemoryHints::default(),
            trace: pixels::wgpu::Trace::Off,
        };

        wasm_bindgen_futures::spawn_local(async move {
            let mut pixels = pixels::PixelsBuilder::new(width, height, surface_texture)
                .wgpu_backend(pixels::wgpu::Backends::GL)
                .device_descriptor(device_descriptor)
                .enable_vsync(true)
                .build_async()
                .await
                .unwrap();
            pixels.set_scaling_mode(ScalingMode::Fill);

            proxy.send_event(AppEvent::PixelsReady(pixels)).unwrap();
        });
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: AppEvent) {
        match event {
            AppEvent::PixelsReady(pixels) => {
                if let Some(window) = self.window.as_ref() {
                    self.egui = Some(EguiState::new(window, &pixels));
                }
                self.pixels = Some(pixels);
                self.resize_scene(self.size_logical.width, self.size_logical.height);
                self.sync_scene_audio();
                self.window.as_ref().unwrap().request_redraw();
            }
            AppEvent::SwitchScene(scene) => {
                self.scene = scene;
                self.start_time = Instant::now();
                self.last_frame_time = self.start_time;
                self.fps_counter.reset();
                self.sync_scene_audio();
                self.window.as_ref().unwrap().request_redraw();
            }
            AppEvent::ResizeScene { width, height } => self.resize_scene(width, height),
            AppEvent::SetAudioEnabled(enabled) => self.set_audio_enabled(enabled),
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent) {
        self.handle_window_event(event_loop, event);
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn audio_base_path() -> &'static str {
    env!("CARGO_MANIFEST_DIR")
}

#[cfg(target_arch = "wasm32")]
fn audio_base_path() -> &'static str {
    ""
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
