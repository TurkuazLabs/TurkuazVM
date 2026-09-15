// # 📄 Dosya Yolu: /turkuazvm/apps/display/src/services/display_service.rs
// # 📌 Amac: Native display session connect ve view lifecycle use-case'ini orkestre eder
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: RFB framebuffer session ve opsiyonel Gaming Input bridge kurup native view'u baslatir
// # Bagimli Oldugu Katman: Tool | View

use crate::config::display_config::DisplayConfig;
use crate::tools::gaming_input_bridge_tool::GamingInputBridgeTool;
use crate::tools::gamepad_input_tool::GamepadInputTool;
use crate::tools::rfb_client_tool::RfbClientTool;
use crate::views::native_display_view::NativeDisplayView;

pub struct DisplayService;

impl DisplayService {
    pub fn run(config: DisplayConfig) -> Result<(), String> {
        let session = RfbClientTool::connect(config.endpoint)?;
        let gaming_input = config.gaming_input.then(|| {
            GamingInputBridgeTool::spawn(
                config.vm_id.clone(),
                config.engine_endpoint,
                config.engine_token.clone(),
            )
        });
        let gamepad = gaming_input
            .as_ref()
            .and_then(|_| GamepadInputTool::new().ok());
        NativeDisplayView::run(config, session, gaming_input, gamepad)
    }
}
