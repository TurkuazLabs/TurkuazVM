// # 📄 Dosya Yolu: /turkuazvm/crates/gaming-input/src/services/gaming_input_translator_service.rs
// # 📌 Amac: Raw host input eventlerini Android-yonlu semantic input planlarina cevirir
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: Keyboard, mouse, gamepad, WASD joystick ve persistent multi-touch state'ini full-frame semantic plana cevirir
// # Bagimli Oldugu Katman: Service

use std::collections::{HashMap, HashSet};

use crate::domain::event::GamingInputEvent;
use crate::domain::plan::{GamingInputPlan, TouchContact, TouchPhase};
use crate::domain::profile::{
    GameInputProfile, GamingKey, InputSource, InputTarget, NormalizedPoint,
};

const MOUSE_DELTA_SCALE_DIVISOR: i32 = 1000;
const GAMEPAD_AXIS_MAX_MILLI: i32 = 1000;
const GAMEPAD_DEADZONE_MILLI: i16 = 120;
const GAMEPAD_LOOK_DIVISOR: i32 = 40;
const AXIS_LEFT_X: u16 = 0;
const AXIS_LEFT_Y: u16 = 1;
const AXIS_RIGHT_X: u16 = 2;
const AXIS_RIGHT_Y: u16 = 3;

#[derive(Debug, Default)]
struct VmInputState {
    pressed_keys: HashSet<GamingKey>,
    joystick_active: bool,
    mouse_capture: bool,
    mouse_look_active: bool,
    mouse_look_position: Option<NormalizedPoint>,
    gamepad_left_x: i16,
    gamepad_left_y: i16,
    active_touches: HashMap<u8, NormalizedPoint>,
}

#[derive(Debug, Default)]
pub struct GamingInputTranslatorService {
    states: HashMap<String, VmInputState>,
}

impl GamingInputTranslatorService {
    pub fn translate(&mut self, profile: &GameInputProfile, event: GamingInputEvent) -> GamingInputPlan {
        let state = self.states.entry(profile.vm_id().as_str().to_owned()).or_default();
        match event {
            GamingInputEvent::Key { key, pressed } => {
                if matches!(key, GamingKey::W | GamingKey::A | GamingKey::S | GamingKey::D) && profile.joystick().enabled() {
                    update_key_state(&mut state.pressed_keys, key, pressed);
                    return joystick_plan(profile, state);
                }
                binding_plan(profile, state, InputSource::Key(key), pressed)
            }
            GamingInputEvent::MouseButton { button, pressed } => binding_plan(profile, state, InputSource::MouseButton(button), pressed),
            GamingInputEvent::MouseCapture { captured } => {
                state.mouse_capture = captured;
                if !captured && state.mouse_look_active {
                    state.mouse_look_active = false;
                    let position = state.mouse_look_position.unwrap_or_else(|| profile.mouse_look().anchor());
                    state.mouse_look_position = None;
                    return touch_frame(state, TouchContact { pointer_id: profile.mouse_look().pointer_id(), phase: TouchPhase::Up, position });
                }
                GamingInputPlan::Noop
            }
            GamingInputEvent::MouseMotion { delta_x, delta_y } => mouse_look_plan(profile, state, delta_x, delta_y),
            GamingInputEvent::GamepadButton { button, pressed } => binding_plan(profile, state, InputSource::GamepadButton(button), pressed),
            GamingInputEvent::GamepadAxis { axis, value_milli } => gamepad_axis_plan(profile, state, axis, value_milli),
        }
    }

    pub fn reset(&mut self, vm_id: &str) { self.states.remove(vm_id); }
}

fn update_key_state(keys: &mut HashSet<GamingKey>, key: GamingKey, pressed: bool) {
    if pressed { keys.insert(key); } else { keys.remove(&key); }
}

fn joystick_plan(profile: &GameInputProfile, state: &mut VmInputState) -> GamingInputPlan {
    let keyboard_x = i32::from(state.pressed_keys.contains(&GamingKey::D)) - i32::from(state.pressed_keys.contains(&GamingKey::A));
    let keyboard_y = i32::from(state.pressed_keys.contains(&GamingKey::S)) - i32::from(state.pressed_keys.contains(&GamingKey::W));
    let (x_milli, y_milli) = if keyboard_x != 0 || keyboard_y != 0 {
        (keyboard_x * GAMEPAD_AXIS_MAX_MILLI, keyboard_y * GAMEPAD_AXIS_MAX_MILLI)
    } else {
        (i32::from(state.gamepad_left_x), i32::from(state.gamepad_left_y))
    };
    joystick_vector_plan(profile, state, x_milli, y_milli)
}

fn joystick_vector_plan(profile: &GameInputProfile, state: &mut VmInputState, x_milli: i32, y_milli: i32) -> GamingInputPlan {
    let center = profile.joystick().center();
    if x_milli.abs() < i32::from(GAMEPAD_DEADZONE_MILLI) && y_milli.abs() < i32::from(GAMEPAD_DEADZONE_MILLI) {
        if !state.joystick_active { return GamingInputPlan::Noop; }
        state.joystick_active = false;
        return touch_frame(state, TouchContact { pointer_id: profile.joystick().pointer_id(), phase: TouchPhase::Up, position: center });
    }
    let radius = i32::from(profile.joystick().radius());
    let position = center.offset_clamped(
        x_milli.clamp(-GAMEPAD_AXIS_MAX_MILLI, GAMEPAD_AXIS_MAX_MILLI) * radius / GAMEPAD_AXIS_MAX_MILLI,
        y_milli.clamp(-GAMEPAD_AXIS_MAX_MILLI, GAMEPAD_AXIS_MAX_MILLI) * radius / GAMEPAD_AXIS_MAX_MILLI,
    );
    let phase = if state.joystick_active { TouchPhase::Move } else { state.joystick_active = true; TouchPhase::Down };
    touch_frame(state, TouchContact { pointer_id: profile.joystick().pointer_id(), phase, position })
}

fn mouse_look_plan(profile: &GameInputProfile, state: &mut VmInputState, delta_x: i32, delta_y: i32) -> GamingInputPlan {
    if !profile.mouse_look().enabled() || !state.mouse_capture || (delta_x == 0 && delta_y == 0) { return GamingInputPlan::Noop; }
    let current = state.mouse_look_position.unwrap_or_else(|| profile.mouse_look().anchor());
    let scaled_x = delta_x.saturating_mul(i32::from(profile.mouse_look().sensitivity_x_milli())) / MOUSE_DELTA_SCALE_DIVISOR;
    let scaled_y = delta_y.saturating_mul(i32::from(profile.mouse_look().sensitivity_y_milli())) / MOUSE_DELTA_SCALE_DIVISOR;
    let position = current.offset_clamped(scaled_x, scaled_y);
    state.mouse_look_position = Some(position);
    let phase = if state.mouse_look_active { TouchPhase::Move } else { state.mouse_look_active = true; TouchPhase::Down };
    touch_frame(state, TouchContact { pointer_id: profile.mouse_look().pointer_id(), phase, position })
}

fn gamepad_axis_plan(profile: &GameInputProfile, state: &mut VmInputState, axis: u16, value_milli: i16) -> GamingInputPlan {
    let value = if value_milli.abs() < GAMEPAD_DEADZONE_MILLI { 0 } else { value_milli };
    match axis {
        AXIS_LEFT_X => { state.gamepad_left_x = value; joystick_plan(profile, state) }
        AXIS_LEFT_Y => { state.gamepad_left_y = value; joystick_plan(profile, state) }
        AXIS_RIGHT_X if state.mouse_capture => mouse_look_plan(profile, state, i32::from(value) / GAMEPAD_LOOK_DIVISOR, 0),
        AXIS_RIGHT_Y if state.mouse_capture => mouse_look_plan(profile, state, 0, i32::from(value) / GAMEPAD_LOOK_DIVISOR),
        _ => GamingInputPlan::Noop,
    }
}

fn binding_plan(profile: &GameInputProfile, state: &mut VmInputState, source: InputSource, pressed: bool) -> GamingInputPlan {
    let Some(binding) = profile.binding_for(source) else { return GamingInputPlan::Noop; };
    match binding.target() {
        InputTarget::Tap { position } if pressed => GamingInputPlan::AndroidTap { position: *position },
        InputTarget::Tap { .. } => GamingInputPlan::Noop,
        InputTarget::AndroidKey { key_code } if pressed => GamingInputPlan::AndroidKey { key_code: *key_code },
        InputTarget::AndroidKey { .. } => GamingInputPlan::Noop,
        InputTarget::HoldTouch { pointer_id, position } => touch_frame(state, TouchContact {
            pointer_id: *pointer_id,
            phase: if pressed { TouchPhase::Down } else { TouchPhase::Up },
            position: *position,
        }),
    }
}

fn touch_frame(state: &mut VmInputState, changed: TouchContact) -> GamingInputPlan {
    let mut contacts = state.active_touches.iter().filter(|(id, _)| **id != changed.pointer_id).map(|(id, position)| TouchContact {
        pointer_id: *id, phase: TouchPhase::Move, position: *position,
    }).collect::<Vec<_>>();
    match changed.phase {
        TouchPhase::Down | TouchPhase::Move => { state.active_touches.insert(changed.pointer_id, changed.position); }
        TouchPhase::Up => { state.active_touches.remove(&changed.pointer_id); }
    }
    contacts.push(changed);
    contacts.sort_by_key(|contact| contact.pointer_id);
    GamingInputPlan::TouchFrame { contacts }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::profile::{GameInputProfile, GamingVmId, MouseLookProfile, NormalizedPoint, VirtualJoystickProfile};
    fn profile() -> GameInputProfile { GameInputProfile::create(
        GamingVmId::parse("android-game").expect("vm"), None,
        VirtualJoystickProfile::create(true, NormalizedPoint::create(2_000, 8_000).expect("center"), 1_000, 0).expect("joystick"),
        MouseLookProfile::create(true, NormalizedPoint::create(7_000, 5_000).expect("look"), 1_000, 1_000, 1).expect("mouse"), Vec::new()).expect("profile") }
    #[test] fn wasd_generates_touch_frame() { let mut service=GamingInputTranslatorService::default(); assert!(matches!(service.translate(&profile(), GamingInputEvent::Key { key: GamingKey::W, pressed: true }), GamingInputPlan::TouchFrame { .. })); }
    #[test] fn mouse_look_requires_capture() { let mut service=GamingInputTranslatorService::default(); assert_eq!(service.translate(&profile(), GamingInputEvent::MouseMotion { delta_x: 5, delta_y: 2 }), GamingInputPlan::Noop); }
    #[test] fn gamepad_left_axis_generates_virtual_joystick_frame() {
        let mut service = GamingInputTranslatorService::default();
        let plan = service.translate(&profile(), GamingInputEvent::GamepadAxis { axis: AXIS_LEFT_X, value_milli: 750 });
        assert!(matches!(plan, GamingInputPlan::TouchFrame { .. }));
    }
}
