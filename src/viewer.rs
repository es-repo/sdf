use crate::fps_counter::FpsCounter;
use font8x8::UnicodeFonts;
use pixels::{Pixels, ScalingMode};
use rayon::prelude::*;
use sdf::audio::{AudioAnalysis, AudioTrack};
use sdf::color_ext::ColorExt;
use sdf::geometry::Vec2;
use sdf::input::InputState;
use sdf::scene::{FrameTime, Scene};
use std::sync::Arc;
use web_time::Instant;
use winit::application::ApplicationHandler;
use winit::dpi::{LogicalSize, PhysicalSize};
use winit::event::{ElementState, MouseScrollDelta, TouchPhase, WindowEvent};
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

const LINE_SCROLL_LOGICAL_PIXELS: f32 = 32.0;
const CONTROLS_WINDOW_TITLE: &str = "Controls";
#[cfg(target_arch = "wasm32")]
const PERSISTED_APP_STATE_KEY: &str = "sdf.app_state";
#[cfg(target_arch = "wasm32")]
const PERSISTED_APP_STATE_VERSION: u32 = 1;
#[cfg(target_arch = "wasm32")]
const STATE_SAVE_INTERVAL_SECONDS: f32 = 0.5;

#[cfg(target_arch = "wasm32")]
#[derive(serde::Serialize, serde::Deserialize)]
struct PersistedAppState {
    version: u32,
    scene_time: f32,
    scene_time_paused: bool,
    scene_state: Option<serde_json::Value>,
}

pub struct Viewer {
    window: Option<Arc<Window>>,
    pixels: Option<Pixels<'static>>,
    scene: Box<dyn Scene>,
    egui: Option<EguiState>,
    #[cfg(target_arch = "wasm32")]
    event_proxy: EventLoopProxy<AppEvent>,
    size_logical: LogicalSize<u32>,
    scale_factor: f64,
    scene_time: f32,
    scene_time_paused: bool,
    last_frame_time: Instant,
    input: InputState,
    touch_input_seen: bool,
    fps_counter: FpsCounter,
    show_fps: bool,
    audio_track: Option<AudioTrack>,
    cached_audio_analysis: AudioAnalysis,
    #[cfg(target_arch = "wasm32")]
    audio_enabled: bool,
    #[cfg(target_arch = "wasm32")]
    last_persisted_state_save: Instant,
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
    pub fn new(size_logical: LogicalSize<u32>, scene: Box<dyn Scene>) -> Self {
        let now = Instant::now();

        Self {
            window: None,
            pixels: None,
            scene,
            egui: None,
            size_logical,
            scene_time: 0.0,
            scene_time_paused: false,
            last_frame_time: now,
            input: InputState::default(),
            touch_input_seen: false,
            fps_counter: FpsCounter::new(),
            show_fps: false,
            scale_factor: 1.0,
            audio_track: None,
            cached_audio_analysis: AudioAnalysis::default(),
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub fn new(size_logical: LogicalSize<u32>, scene: Box<dyn Scene>, event_proxy: EventLoopProxy<AppEvent>) -> Self {
        let now = Instant::now();

        Self {
            window: None,
            pixels: None,
            scene,
            egui: None,
            event_proxy,
            size_logical,
            scene_time: 0.0,
            scene_time_paused: false,
            last_frame_time: now,
            input: InputState::default(),
            touch_input_seen: false,
            fps_counter: FpsCounter::new(),
            show_fps: false,
            scale_factor: 1.0,
            audio_track: None,
            cached_audio_analysis: AudioAnalysis::default(),
            audio_enabled: false,
            last_persisted_state_save: now,
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
        if self.scene_time_paused {
            return self.cached_audio_analysis;
        }

        self.cached_audio_analysis = self.audio_track.as_mut().map(AudioTrack::analysis).unwrap_or_default();
        self.cached_audio_analysis
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

    fn update_scene_time_paused(&mut self, paused: bool) {
        self.scene_time_paused = paused;

        #[cfg(target_arch = "wasm32")]
        crate::wasm_boot::store_scene_time_paused(paused);
    }

    fn set_scene_time_paused(&mut self, paused: bool) {
        self.update_scene_time_paused(paused);
        self.save_persisted_state();

        if let Some(window) = self.window.as_ref() {
            window.request_redraw();
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn reset_scene_time(&mut self) {
        self.scene_time = 0.0;
        self.last_frame_time = Instant::now();
        self.save_persisted_state();

        if let Some(window) = self.window.as_ref() {
            window.request_redraw();
        }
    }

    fn prepare_egui_frame(&mut self) -> Option<EguiFrame> {
        if !self.scene.has_controls_ui() {
            return None;
        }

        let touch_input_seen = self.touch_input_seen;
        let scene = self.scene.as_mut();
        let window = self.window.as_ref()?;
        let window_surface_size = window.inner_size();

        #[cfg(target_arch = "wasm32")]
        let surface_size = {
            let pixels = self.pixels.as_ref()?;
            web_surface_size(
                window_surface_size,
                pixels.context().device.limits().max_texture_dimension_2d,
            )
        };
        #[cfg(not(target_arch = "wasm32"))]
        let surface_size = {
            // Device-pixel sizes can truncate fractional logical pixels while
            // LogicalSize::to_physical can round them up. Keep egui's scissor
            // rectangles no larger than the actual render target.
            let expected_surface_size = self.size_logical.to_physical::<u32>(self.scale_factor);
            PhysicalSize::new(
                expected_surface_size.width.min(window_surface_size.width),
                expected_surface_size.height.min(window_surface_size.height),
            )
        };

        let egui = self.egui.as_mut()?;
        let mut raw_input = egui.state.take_egui_input(window);
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
            compact_style.spacing.interact_size.y = if touch_input_seen { 32.0 } else { 16.0 };
            compact_style.spacing.window_margin = egui::Margin::symmetric(6, 2);
            let compact_frame = egui::Frame::window(&compact_style).shadow(egui::Shadow::NONE);
            context.set_style(compact_style);

            let controls_window_response = egui::Window::new(CONTROLS_WINDOW_TITLE)
                .anchor(egui::Align2::RIGHT_TOP, egui::vec2(0.0, 0.0))
                .default_width(220.0)
                .frame(compact_frame)
                .resizable(false)
                .collapsible(true)
                .default_open(false)
                .show(context, |ui| {
                    scene.controls_ui(ui);
                });

            if controls_window_response
                .as_ref()
                .is_some_and(|response| response.response.clicked_elsewhere())
                && !context.is_pointer_over_area()
            {
                collapse_controls_window(context);
            }

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
        let real_time_delta = now.duration_since(self.last_frame_time).as_secs_f32();
        self.last_frame_time = now;

        let scene_time_delta = if self.scene_time_paused { 0.0 } else { real_time_delta };
        self.scene_time += scene_time_delta;

        let scene_time = self.scene_time;
        let frame_time = FrameTime {
            real_time_delta,
            scene_time_delta,
            scene_time,
        };
        self.fps_counter.tick();

        #[cfg(target_arch = "wasm32")]
        crate::wasm_boot::store_playback_metrics(scene_time, self.fps_counter.count());

        let egui_frame = self.prepare_egui_frame();
        self.sync_audio_volume();
        let audio_analysis = self.audio_analysis();
        self.scene.update(frame_time, &self.input);
        self.input.reset_frame_delta();
        let prepared_scene = self.scene.prepare_frame_with_audio(scene_time, &audio_analysis);
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
                let color = prepared_scene.get_pixel_color_with_audio(coord, scene_time, &audio_analysis);
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
        self.maybe_save_persisted_state();

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

                    if !gui_consumed && matches!(event.physical_key, PhysicalKey::Code(KeyCode::Space)) {
                        self.set_scene_time_paused(!self.scene_time_paused);
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
                    .set_pointer_position(Vec2::new(position.x as f32, position.y as f32));
            }

            WindowEvent::MouseWheel { delta, .. } => {
                if !gui_consumed {
                    self.input.add_scroll_delta(self.scroll_delta(delta));
                }
            }

            WindowEvent::Touch(touch) => {
                self.touch_input_seen = true;
                let position = Vec2::new(touch.location.x as f32, touch.location.y as f32);

                match touch.phase {
                    TouchPhase::Started if !gui_consumed => {
                        self.input.start_touch(touch.id, position);
                        self.sync_scene_audio();
                    }
                    TouchPhase::Moved if !gui_consumed => {
                        self.input.move_touch(touch.id, position, self.scale_factor as f32);
                    }
                    TouchPhase::Ended | TouchPhase::Cancelled => self.input.end_touch(touch.id),
                    _ => {}
                }
            }

            WindowEvent::CursorLeft { .. } => self.input.clear_pointer_position(),

            WindowEvent::Focused(false) => self.input.clear(),

            WindowEvent::Resized(size_physical) => {
                if size_physical.width > 0 && size_physical.height > 0 {
                    let Some(pixels) = self.pixels.as_mut() else {
                        #[cfg(not(target_arch = "wasm32"))]
                        {
                            self.size_logical = size_physical.to_logical::<u32>(self.scale_factor);
                        }
                        return;
                    };

                    #[cfg(target_arch = "wasm32")]
                    let surface_size = web_surface_size(
                        size_physical,
                        pixels.context().device.limits().max_texture_dimension_2d,
                    );
                    #[cfg(not(target_arch = "wasm32"))]
                    let surface_size = size_physical;

                    pixels
                        .resize_surface(surface_size.width, surface_size.height)
                        .unwrap();

                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        self.size_logical = size_physical.to_logical::<u32>(self.scale_factor);
                        pixels
                            .resize_buffer(self.size_logical.width, self.size_logical.height)
                            .unwrap();
                    }

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
        if !self.scene.has_controls_ui() {
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

    fn scroll_delta(&self, delta: MouseScrollDelta) -> Vec2 {
        match delta {
            MouseScrollDelta::LineDelta(x, y) => Vec2::new(x, y) * LINE_SCROLL_LOGICAL_PIXELS,
            MouseScrollDelta::PixelDelta(position) => {
                let scale_factor = self.scale_factor as f32;
                Vec2::new(position.x as f32 / scale_factor, position.y as f32 / scale_factor)
            }
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn resize_scene(&mut self, width: u32, height: u32, surface_size: Option<PhysicalSize<u32>>) {
        if width == 0 || height == 0 {
            return;
        }

        self.size_logical = LogicalSize::new(width, height);

        let Some(window) = self.window.as_ref() else {
            return;
        };

        if let Some(pixels) = self.pixels.as_mut() {
            let surface_size = web_surface_size(
                surface_size.unwrap_or_else(|| window.inner_size()),
                pixels.context().device.limits().max_texture_dimension_2d,
            );

            if surface_size.width > 0 && surface_size.height > 0 {
                pixels
                    .resize_surface(surface_size.width, surface_size.height)
                    .unwrap();
            }
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
        self.scene_time = 0.0;
        self.update_scene_time_paused(false);
        self.last_frame_time = Instant::now();
        self.cached_audio_analysis = AudioAnalysis::default();
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

#[cfg(target_arch = "wasm32")]
impl Viewer {
    fn restore_persisted_state(&mut self) {
        let Some(storage) = browser_storage() else {
            return;
        };
        let Some(serialized) = storage.get_item(PERSISTED_APP_STATE_KEY).ok().flatten() else {
            return;
        };
        let Ok(state) = serde_json::from_str::<PersistedAppState>(&serialized) else {
            let _ = storage.remove_item(PERSISTED_APP_STATE_KEY);
            return;
        };

        if state.version != PERSISTED_APP_STATE_VERSION {
            let _ = storage.remove_item(PERSISTED_APP_STATE_KEY);
            return;
        }

        self.scene_time = if state.scene_time.is_finite() {
            state.scene_time.max(0.0)
        } else {
            0.0
        };
        self.update_scene_time_paused(state.scene_time_paused);

        if let Some(scene_state) = state.scene_state.as_ref() {
            self.scene.load_state(scene_state);
        }

        self.last_persisted_state_save = Instant::now();
    }

    fn maybe_save_persisted_state(&mut self) {
        let now = Instant::now();

        if now.duration_since(self.last_persisted_state_save).as_secs_f32() < STATE_SAVE_INTERVAL_SECONDS {
            return;
        }

        self.save_persisted_state();
        self.last_persisted_state_save = now;
    }

    fn save_persisted_state(&self) {
        let state = PersistedAppState {
            version: PERSISTED_APP_STATE_VERSION,
            scene_time: self.scene_time,
            scene_time_paused: self.scene_time_paused,
            scene_state: self.scene.save_state(),
        };

        let Some(storage) = browser_storage() else {
            return;
        };
        let Ok(serialized) = serde_json::to_string(&state) else {
            return;
        };

        let _ = storage.set_item(PERSISTED_APP_STATE_KEY, &serialized);
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl Viewer {
    fn maybe_save_persisted_state(&mut self) {}

    fn save_persisted_state(&self) {}
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
        self.restore_persisted_state();
        let surface_texture = self.create_surface_texture();
        let proxy = self.event_proxy.clone();
        let width = self.size_logical.width;
        let height = self.size_logical.height;

        wasm_bindgen_futures::spawn_local(async move {
            let mut pixels = pixels::PixelsBuilder::new(width, height, surface_texture)
                .wgpu_backend(pixels::wgpu::Backends::GL)
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
                self.resize_scene(self.size_logical.width, self.size_logical.height, None);
                self.sync_scene_audio();
                self.window.as_ref().unwrap().request_redraw();
            }
            AppEvent::SwitchScene(scene) => {
                self.scene = scene;
                self.scene_time = 0.0;
                self.update_scene_time_paused(false);
                self.last_frame_time = Instant::now();
                self.cached_audio_analysis = AudioAnalysis::default();
                self.fps_counter.reset();
                self.save_persisted_state();
                self.sync_scene_audio();
                self.window.as_ref().unwrap().request_redraw();
            }
            AppEvent::ResizeScene {
                width,
                height,
                surface_width,
                surface_height,
            } => self.resize_scene(
                width,
                height,
                Some(PhysicalSize::new(surface_width, surface_height)),
            ),
            AppEvent::SetAudioEnabled(enabled) => self.set_audio_enabled(enabled),
            AppEvent::ResetSceneTime => self.reset_scene_time(),
            AppEvent::SetSceneTimePaused(paused) => self.set_scene_time_paused(paused),
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

fn collapse_controls_window(context: &egui::Context) {
    let id = egui::Id::new(CONTROLS_WINDOW_TITLE).with("collapsing");
    let mut state = egui::collapsing_header::CollapsingState::load_with_default_open(context, id, false);

    if state.is_open() {
        state.set_open(false);
        state.store(context);
        context.request_repaint();
    }
}

#[cfg(target_arch = "wasm32")]
fn web_surface_size(size: PhysicalSize<u32>, max_dimension: u32) -> PhysicalSize<u32> {
    if size.width == 0 || size.height == 0 {
        return size;
    }

    let scale = (max_dimension as f64 / size.width as f64)
        .min(max_dimension as f64 / size.height as f64)
        .min(1.0);

    PhysicalSize::new(
        (size.width as f64 * scale).round().max(1.0) as u32,
        (size.height as f64 * scale).round().max(1.0) as u32,
    )
}

#[cfg(target_arch = "wasm32")]
fn browser_storage() -> Option<web_sys::Storage> {
    web_sys::window()?.local_storage().ok().flatten()
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
