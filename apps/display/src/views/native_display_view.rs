// # 📄 Dosya Yolu: /turkuazvm/apps/display/src/views/native_display_view.rs
// # 📌 Amac: RFB framebufferini native pencereye cizer ve kullanici inputunu session'a aktarir
// # 📌 Modul - Rust
// # Version: 0.40.2
// # Aciklama: Winit/Softbuffer fallback renderer, cached nearest-neighbor scale map, gercek render suresi ve ayrik RFB/present telemetrisini uygular
// # Bagimli Oldugu Katman: Tool | View

use std::num::NonZeroU32;
use std::rc::Rc;
use std::time::Instant;

use softbuffer::{Context, Surface};
use turkuazvm_engine_api::GamingInputEventDto;
use turkuazvm_gaming_input::ports::host_gamepad_port::HostGamepadPort;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{DeviceEvent, ElementState, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop, OwnedDisplayHandle};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{CursorGrabMode, Fullscreen, Window, WindowId};

use crate::config::display_config::DisplayConfig;
use crate::tools::frame_clock_tool::FrameClockTool;
use crate::tools::scale_map_tool::ScaleMapTool;
use crate::tools::gaming_input_bridge_tool::GamingInputBridgeTool;
use crate::tools::gamepad_input_tool::GamepadInputTool;
use crate::tools::gaming_keymap_tool;
use crate::tools::rfb_client_tool::{FrameSnapshot, RfbClientMessage, RfbSession};
use crate::tools::rfb_keysym_tool;

const INITIAL_WINDOW_WIDTH: f64 = 1280.0;
const INITIAL_WINDOW_HEIGHT: f64 = 720.0;
const BUTTON_LEFT: u8 = 1;
const BUTTON_MIDDLE: u8 = 2;
const BUTTON_RIGHT: u8 = 4;
const BUTTON_WHEEL_UP: u8 = 8;
const BUTTON_WHEEL_DOWN: u8 = 16;
const WHEEL_THRESHOLD: f64 = 0.0;
const HOST_WIDTH_CONVERSION_ERROR: &str = "HOST_WIDTH_CONVERSION_FAILED";
const HOST_HEIGHT_CONVERSION_ERROR: &str = "HOST_HEIGHT_CONVERSION_FAILED";

pub struct NativeDisplayView;

impl NativeDisplayView {
    pub fn run(
        config: DisplayConfig,
        session: RfbSession,
        gaming_input: Option<GamingInputBridgeTool>,
        gamepad: Option<GamepadInputTool>,
    ) -> Result<(), String> {
        let event_loop = EventLoop::new().map_err(|error| error.to_string())?;
        event_loop.set_control_flow(ControlFlow::Poll);
        let context = Context::new(event_loop.owned_display_handle())
            .map_err(|error| error.to_string())?;
        let mut app = NativeDisplayApp::new(config, session, context, gaming_input, gamepad);
        event_loop.run_app(&mut app).map_err(|error| error.to_string())
    }
}

struct NativeDisplayApp {
    config: DisplayConfig,
    session: RfbSession,
    gaming_input: Option<GamingInputBridgeTool>,
    gamepad: Option<GamepadInputTool>,
    context: Context<OwnedDisplayHandle>,
    surface: Option<Surface<OwnedDisplayHandle, Rc<Window>>>,
    frame: FrameSnapshot,
    frame_dirty: bool,
    fullscreen: bool,
    pointer_captured: bool,
    pointer_x: f64,
    pointer_y: f64,
    button_mask: u8,
    clock: FrameClockTool,
    scale_map: ScaleMapTool,
    render_target_size: Option<(u32, u32)>,
}

impl NativeDisplayApp {
    fn new(
        config: DisplayConfig,
        session: RfbSession,
        context: Context<OwnedDisplayHandle>,
        gaming_input: Option<GamingInputBridgeTool>,
        gamepad: Option<GamepadInputTool>,
    ) -> Self {
        let width = session.initial_width;
        let height = session.initial_height;
        Self {
            fullscreen: config.fullscreen,
            config,
            session,
            gaming_input,
            gamepad,
            context,
            surface: None,
            frame: FrameSnapshot {
                width,
                height,
                pixels: vec![0_u32; usize::from(width) * usize::from(height)],
            },
            frame_dirty: true,
            pointer_captured: false,
            pointer_x: f64::from(width) / 2.0,
            pointer_y: f64::from(height) / 2.0,
            button_mask: 0,
            clock: FrameClockTool::new(),
            scale_map: ScaleMapTool::default(),
            render_target_size: None,
        }
    }

    fn window_handle(&self) -> Option<Rc<Window>> {
        self.surface.as_ref().map(|surface| surface.window().clone())
    }

    fn update_latest_frame(&mut self) {
        while let Ok(frame) = self.session.frame_receiver.try_recv() {
            self.frame = frame;
            self.frame_dirty = true;
        }
    }

    fn toggle_fullscreen(&mut self) {
        self.fullscreen = !self.fullscreen;
        if let Some(window) = self.window_handle() {
            let mode = if self.fullscreen {
                Some(Fullscreen::Borderless(window.current_monitor()))
            } else {
                None
            };
            window.set_fullscreen(mode);
        }
    }

    fn set_pointer_capture(&mut self, enabled: bool) {
        let Some(window) = self.window_handle() else {
            return;
        };
        if enabled {
            let grabbed = window
                .set_cursor_grab(CursorGrabMode::Locked)
                .or_else(|_| window.set_cursor_grab(CursorGrabMode::Confined))
                .is_ok();
            self.pointer_captured = grabbed;
            window.set_cursor_visible(!grabbed);
        } else {
            let _ = window.set_cursor_grab(CursorGrabMode::None);
            window.set_cursor_visible(true);
            self.pointer_captured = false;
        }
        if let Some(bridge) = &self.gaming_input {
            bridge.try_send(GamingInputEventDto::MouseCapture {
                captured: self.pointer_captured,
            });
        }
    }

    fn toggle_pointer_capture(&mut self) {
        self.set_pointer_capture(!self.pointer_captured);
    }

    fn send_key(&self, code: KeyCode, down: bool) {
        if self.pointer_captured {
            if let (Some(bridge), Some(key)) = (&self.gaming_input, gaming_keymap_tool::key(code)) {
                bridge.try_send(GamingInputEventDto::Key { key, pressed: down });
                return;
            }
        }
        if let Some(keysym) = rfb_keysym_tool::keysym(code) {
            let _ = self.session.send(RfbClientMessage::Key { keysym, down });
        }
    }

    fn update_pointer_from_window(&mut self, x: f64, y: f64) {
        let Some(window) = self.window_handle() else {
            return;
        };
        let size = window.inner_size();
        if size.width == 0 || size.height == 0 {
            return;
        }
        self.pointer_x = (x / f64::from(size.width) * f64::from(self.frame.width))
            .clamp(0.0, f64::from(self.frame.width.saturating_sub(1)));
        self.pointer_y = (y / f64::from(size.height) * f64::from(self.frame.height))
            .clamp(0.0, f64::from(self.frame.height.saturating_sub(1)));
        self.send_pointer();
    }

    fn update_pointer_from_delta(&mut self, delta_x: f64, delta_y: f64) {
        if !self.pointer_captured {
            return;
        }
        self.pointer_x = (self.pointer_x + delta_x)
            .clamp(0.0, f64::from(self.frame.width.saturating_sub(1)));
        self.pointer_y = (self.pointer_y + delta_y)
            .clamp(0.0, f64::from(self.frame.height.saturating_sub(1)));
        self.send_pointer();
    }

    fn send_pointer(&self) {
        let _ = self.session.send(RfbClientMessage::Pointer {
            button_mask: self.button_mask,
            x: self.pointer_x.round() as u16,
            y: self.pointer_y.round() as u16,
        });
    }

    fn set_mouse_button(&mut self, button: MouseButton, down: bool) {
        if self.pointer_captured {
            if let (Some(bridge), Some(button)) = (&self.gaming_input, gaming_keymap_tool::mouse_button(button)) {
                bridge.try_send(GamingInputEventDto::MouseButton { button, pressed: down });
                return;
            }
        }
        let mask = match button {
            MouseButton::Left => BUTTON_LEFT,
            MouseButton::Middle => BUTTON_MIDDLE,
            MouseButton::Right => BUTTON_RIGHT,
            _ => return,
        };
        if down {
            self.button_mask |= mask;
        } else {
            self.button_mask &= !mask;
        }
        self.send_pointer();
    }

    fn send_wheel(&self, delta: &MouseScrollDelta) {
        let direction = match delta {
            MouseScrollDelta::LineDelta(_, y) => f64::from(*y),
            MouseScrollDelta::PixelDelta(position) => position.y,
        };
        if direction.abs() <= WHEEL_THRESHOLD {
            return;
        }
        let wheel_mask = if direction > WHEEL_THRESHOLD {
            BUTTON_WHEEL_UP
        } else {
            BUTTON_WHEEL_DOWN
        };
        let _ = self.session.send(RfbClientMessage::Pointer {
            button_mask: self.button_mask | wheel_mask,
            x: self.pointer_x.round() as u16,
            y: self.pointer_y.round() as u16,
        });
        self.send_pointer();
    }

    fn poll_gamepad(&mut self) {
        let (Some(gamepad), Some(bridge)) = (self.gamepad.as_mut(), self.gaming_input.as_ref()) else {
            return;
        };
        if let Ok(events) = gamepad.poll_events() {
            for event in events {
                bridge.try_send_gamepad_event(event);
            }
        }
    }

    fn render(&mut self) -> Result<(), String> {
        let render_started = Instant::now();
        let Some(surface) = self.surface.as_mut() else {
            return Ok(());
        };
        let size = surface.window().inner_size();
        let (Some(width), Some(height)) = (NonZeroU32::new(size.width), NonZeroU32::new(size.height))
        else {
            return Ok(());
        };
        if self.render_target_size != Some((size.width, size.height)) {
            surface.resize(width, height).map_err(|error| error.to_string())?;
            self.render_target_size = Some((size.width, size.height));
        }
        let mut buffer = surface.buffer_mut().map_err(|error| error.to_string())?;
        let guest_width = usize::from(self.frame.width);
        let guest_height = usize::from(self.frame.height);
        let host_width = usize::try_from(size.width)
            .map_err(|_| String::from(HOST_WIDTH_CONVERSION_ERROR))?;
        let host_height = usize::try_from(size.height)
            .map_err(|_| String::from(HOST_HEIGHT_CONVERSION_ERROR))?;

        if guest_width == host_width && guest_height == host_height {
            buffer.copy_from_slice(&self.frame.pixels);
        } else {
            self.scale_map
                .ensure(guest_width, guest_height, host_width, host_height);
            let x_map = self.scale_map.x_map();
            let y_map = self.scale_map.y_map();
            for (host_y, source_y) in y_map.iter().copied().enumerate() {
                let host_row = host_y * host_width;
                let guest_row = source_y * guest_width;
                for (host_x, source_x) in x_map.iter().copied().enumerate() {
                    buffer[host_row + host_x] = self.frame.pixels[guest_row + source_x];
                }
            }
        }
        buffer.present().map_err(|error| error.to_string())?;
        self.clock.record_present(render_started.elapsed());
        self.frame_dirty = false;
        Ok(())
    }

    fn update_title(&mut self) {
        let telemetry = self.session.telemetry_snapshot();
        let Some(sample) = self.clock.sample(telemetry) else {
            return;
        };
        if let Some(window) = self.window_handle() {
            window.set_title(&format!(
                "{} - {} | RFB {:.1} UPS / {:.1} Mbps | Present {:.1} FPS | Render {:.2} ms | Drop {}",
                self.config.title,
                self.config.vm_id,
                sample.rfb_updates_per_second,
                sample.rfb_megabits_per_second,
                sample.present_fps,
                sample.average_render_time_ms,
                sample.dropped_frames
            ));
        }
    }
}

impl ApplicationHandler for NativeDisplayApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.surface.is_some() {
            return;
        }
        let attributes = Window::default_attributes()
            .with_title(format!("{} - {}", self.config.title, self.config.vm_id))
            .with_inner_size(LogicalSize::new(INITIAL_WINDOW_WIDTH, INITIAL_WINDOW_HEIGHT));
        let Ok(window) = event_loop.create_window(attributes) else {
            event_loop.exit();
            return;
        };
        let window = Rc::new(window);
        if self.fullscreen {
            window.set_fullscreen(Some(Fullscreen::Borderless(window.current_monitor())));
        }
        match Surface::new(&self.context, window) {
            Ok(surface) => self.surface = Some(surface),
            Err(_) => event_loop.exit(),
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        if self.window_handle().map(|window| window.id()) != Some(window_id) {
            return;
        }
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Focused(false) => self.set_pointer_capture(false),
            WindowEvent::RedrawRequested => {
                if self.render().is_err() {
                    event_loop.exit();
                }
            }
            WindowEvent::Resized(_) => {
                self.render_target_size = None;
                self.frame_dirty = true;
            }
            WindowEvent::CursorMoved { position, .. } if !self.pointer_captured => {
                self.update_pointer_from_window(position.x, position.y);
            }
            WindowEvent::MouseInput { state, button, .. } => {
                self.set_mouse_button(button, state == ElementState::Pressed);
            }
            WindowEvent::MouseWheel { delta, .. } => self.send_wheel(&delta),
            WindowEvent::KeyboardInput { event, .. } => {
                let PhysicalKey::Code(code) = event.physical_key else {
                    return;
                };
                let down = event.state == ElementState::Pressed;
                if down && !event.repeat && code == KeyCode::F1 {
                    self.toggle_pointer_capture();
                    return;
                }
                if down && !event.repeat && code == KeyCode::F11 {
                    self.toggle_fullscreen();
                    return;
                }
                if down && !event.repeat && code == KeyCode::Escape && self.pointer_captured {
                    self.set_pointer_capture(false);
                    return;
                }
                self.send_key(code, down);
            }
            _ => {}
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: winit::event::DeviceId,
        event: DeviceEvent,
    ) {
        if let DeviceEvent::MouseMotion { delta } = event {
            if self.pointer_captured {
                if let Some(bridge) = &self.gaming_input {
                    bridge.try_send(GamingInputEventDto::MouseMotion {
                        delta_x: delta.0.round().clamp(f64::from(i32::MIN), f64::from(i32::MAX)) as i32,
                        delta_y: delta.1.round().clamp(f64::from(i32::MIN), f64::from(i32::MAX)) as i32,
                    });
                    return;
                }
            }
            self.update_pointer_from_delta(delta.0, delta.1);
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        self.poll_gamepad();
        self.update_latest_frame();
        self.update_title();
        if self.frame_dirty {
            if let Some(window) = self.window_handle() {
                window.request_redraw();
            }
        }
    }
}
