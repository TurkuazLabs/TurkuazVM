// # 📄 Dosya Yolu: /turkuazvm/apps/engine/src/services/engine_auth_service.rs
// # 📌 Amac: Engine API token authentication kuralini uygular
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: Local optional ve remote mandatory token dogrulamasini sabit-zamana yakin byte comparison ile yapar
// # Bagimli Oldugu Katman: Service

#[derive(Debug, Clone)]
pub struct EngineAuthService {
    expected_token: Option<String>,
}

impl EngineAuthService {
    pub fn new(expected_token: Option<String>) -> Self {
        Self { expected_token }
    }

    pub fn authentication_required(&self) -> bool {
        self.expected_token.is_some()
    }

    pub fn authorize(&self, provided_token: Option<&str>) -> bool {
        match (&self.expected_token, provided_token) {
            (None, _) => true,
            (Some(_), None) => false,
            (Some(expected), Some(provided)) => constant_time_equal(expected.as_bytes(), provided.as_bytes()),
        }
    }
}

fn constant_time_equal(left: &[u8], right: &[u8]) -> bool {
    if left.len() != right.len() {
        return false;
    }
    let mut difference = 0_u8;
    for (left_byte, right_byte) in left.iter().zip(right.iter()) {
        difference |= left_byte ^ right_byte;
    }
    difference == 0
}

#[cfg(test)]
mod tests {
    use super::EngineAuthService;

    const TOKEN: &str = "0123456789abcdef0123456789abcdef";

    #[test]
    fn no_auth_configuration_accepts_local_request() {
        assert!(EngineAuthService::new(None).authorize(None));
    }

    #[test]
    fn token_configuration_rejects_missing_or_wrong_token() {
        let service = EngineAuthService::new(Some(TOKEN.to_owned()));
        assert!(!service.authorize(None));
        assert!(!service.authorize(Some("wrong-token")));
        assert!(service.authorize(Some(TOKEN)));
    }
}
