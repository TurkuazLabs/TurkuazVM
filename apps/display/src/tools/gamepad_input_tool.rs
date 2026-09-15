// # 📄 Dosya Yolu: /turkuazvm/apps/display/src/tools/gamepad_input_tool.rs
// # 📌 Amac: Windows ve Linux host gamepad eventlerini Gaming Input domain eventlerine adapte eder
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: GilRs hotplug/input eventlerini TurkuazVM normalize button ve axis kimliklerine cevirir
// # Bagimli Oldugu Katman: Tool

use gilrs::{Axis, Button, EventType, Gilrs};
use turkuazvm_gaming_input::domain::event::GamingInputEvent;
use turkuazvm_gaming_input::ports::host_gamepad_port::{HostGamepadPort, HostGamepadPortError};

const BUTTON_SOUTH: u16 = 0;
const BUTTON_EAST: u16 = 1;
const BUTTON_NORTH: u16 = 2;
const BUTTON_WEST: u16 = 3;
const BUTTON_LEFT_TRIGGER: u16 = 4;
const BUTTON_RIGHT_TRIGGER: u16 = 5;
const BUTTON_LEFT_TRIGGER_2: u16 = 6;
const BUTTON_RIGHT_TRIGGER_2: u16 = 7;
const BUTTON_SELECT: u16 = 8;
const BUTTON_START: u16 = 9;
const BUTTON_MODE: u16 = 10;
const BUTTON_LEFT_THUMB: u16 = 11;
const BUTTON_RIGHT_THUMB: u16 = 12;
const BUTTON_DPAD_UP: u16 = 13;
const BUTTON_DPAD_DOWN: u16 = 14;
const BUTTON_DPAD_LEFT: u16 = 15;
const BUTTON_DPAD_RIGHT: u16 = 16;

const AXIS_LEFT_X: u16 = 0;
const AXIS_LEFT_Y: u16 = 1;
const AXIS_RIGHT_X: u16 = 2;
const AXIS_RIGHT_Y: u16 = 3;
const AXIS_VALUE_SCALE: f32 = 1000.0;
const AXIS_VALUE_MIN: f32 = -1000.0;
const AXIS_VALUE_MAX: f32 = 1000.0;

pub struct GamepadInputTool {
    gilrs: Gilrs,
}

impl GamepadInputTool {
    pub fn new() -> Result<Self, HostGamepadPortError> {
        let gilrs = Gilrs::new().map_err(|error| HostGamepadPortError::Observation(error.to_string()))?;
        Ok(Self { gilrs })
    }
}

impl HostGamepadPort for GamepadInputTool {
    fn available(&self) -> bool {
        true
    }

    fn poll_events(&mut self) -> Result<Vec<GamingInputEvent>, HostGamepadPortError> {
        let mut events = Vec::new();
        while let Some(event) = self.gilrs.next_event() {
            match event.event {
                EventType::ButtonPressed(button, _) => {
                    if let Some(button) = normalize_button(button) {
                        events.push(GamingInputEvent::GamepadButton { button, pressed: true });
                    }
                }
                EventType::ButtonReleased(button, _) => {
                    if let Some(button) = normalize_button(button) {
                        events.push(GamingInputEvent::GamepadButton { button, pressed: false });
                    }
                }
                EventType::AxisChanged(axis, value, _) => {
                    if let Some(axis) = normalize_axis(axis) {
                        events.push(GamingInputEvent::GamepadAxis {
                            axis,
                            value_milli: normalize_axis_value(value),
                        });
                    }
                }
                _ => {}
            }
        }
        Ok(events)
    }
}

fn normalize_button(button: Button) -> Option<u16> {
    match button {
        Button::South => Some(BUTTON_SOUTH),
        Button::East => Some(BUTTON_EAST),
        Button::North => Some(BUTTON_NORTH),
        Button::West => Some(BUTTON_WEST),
        Button::LeftTrigger => Some(BUTTON_LEFT_TRIGGER),
        Button::RightTrigger => Some(BUTTON_RIGHT_TRIGGER),
        Button::LeftTrigger2 => Some(BUTTON_LEFT_TRIGGER_2),
        Button::RightTrigger2 => Some(BUTTON_RIGHT_TRIGGER_2),
        Button::Select => Some(BUTTON_SELECT),
        Button::Start => Some(BUTTON_START),
        Button::Mode => Some(BUTTON_MODE),
        Button::LeftThumb => Some(BUTTON_LEFT_THUMB),
        Button::RightThumb => Some(BUTTON_RIGHT_THUMB),
        Button::DPadUp => Some(BUTTON_DPAD_UP),
        Button::DPadDown => Some(BUTTON_DPAD_DOWN),
        Button::DPadLeft => Some(BUTTON_DPAD_LEFT),
        Button::DPadRight => Some(BUTTON_DPAD_RIGHT),
        _ => None,
    }
}

fn normalize_axis(axis: Axis) -> Option<u16> {
    match axis {
        Axis::LeftStickX => Some(AXIS_LEFT_X),
        Axis::LeftStickY => Some(AXIS_LEFT_Y),
        Axis::RightStickX => Some(AXIS_RIGHT_X),
        Axis::RightStickY => Some(AXIS_RIGHT_Y),
        _ => None,
    }
}

fn normalize_axis_value(value: f32) -> i16 {
    (value * AXIS_VALUE_SCALE)
        .round()
        .clamp(AXIS_VALUE_MIN, AXIS_VALUE_MAX) as i16
}
