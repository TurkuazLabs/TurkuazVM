// # 📄 Dosya Yolu: /turkuazvm/crates/android/src/domain/input.rs
// # 📌 Amac: Android input injection domain komutlarini tanimlar
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: Touch, swipe, key ve guvenli text girdilerini ADB shell syntaxindan bagimsiz typed modellerle tasir
// # Bagimli Oldugu Katman: Service | Tool

const MAX_TEXT_LENGTH: usize = 256;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AndroidInputAction {
    Tap { x: u32, y: u32 },
    Swipe {
        from_x: u32,
        from_y: u32,
        to_x: u32,
        to_y: u32,
        duration_ms: u32,
    },
    KeyEvent { key_code: u32 },
    Text { value: String },
}

impl AndroidInputAction {
    pub fn validate(self) -> Result<Self, AndroidInputError> {
        match &self {
            Self::Swipe { duration_ms, .. } if *duration_ms == 0 || *duration_ms > 60_000 => {
                Err(AndroidInputError::InvalidSwipeDuration)
            }
            Self::KeyEvent { key_code } if *key_code == 0 => Err(AndroidInputError::InvalidKeyCode),
            Self::Text { value } if !is_safe_input_text(value) => {
                Err(AndroidInputError::InvalidText)
            }
            _ => Ok(self),
        }
    }

    pub fn validate_bounds(&self, width: u32, height: u32) -> Result<(), AndroidInputError> {
        let in_bounds = |x: u32, y: u32| x < width && y < height;
        match self {
            Self::Tap { x, y } if !in_bounds(*x, *y) => {
                Err(AndroidInputError::CoordinateOutOfBounds)
            }
            Self::Swipe {
                from_x,
                from_y,
                to_x,
                to_y,
                ..
            } if !in_bounds(*from_x, *from_y) || !in_bounds(*to_x, *to_y) => {
                Err(AndroidInputError::CoordinateOutOfBounds)
            }
            _ => Ok(()),
        }
    }
}

fn is_safe_input_text(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_TEXT_LENGTH
        && value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric()
                || matches!(byte, b' ' | b'.' | b'_' | b'-' | b'@' | b',' | b':')
        })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AndroidInputError {
    InvalidSwipeDuration,
    InvalidKeyCode,
    InvalidText,
    CoordinateOutOfBounds,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shell_metacharacters_are_rejected_from_text_input() {
        let action = AndroidInputAction::Text {
            value: String::from("hello;reboot"),
        };
        assert_eq!(action.validate(), Err(AndroidInputError::InvalidText));
    }

    #[test]
    fn tap_outside_display_is_rejected() {
        let action = AndroidInputAction::Tap { x: 1920, y: 10 };
        assert_eq!(
            action.validate_bounds(1920, 1080),
            Err(AndroidInputError::CoordinateOutOfBounds)
        );
    }
}
