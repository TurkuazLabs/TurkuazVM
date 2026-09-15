// # 📄 Dosya Yolu: /turkuazvm/apps/engine/src/main.rs
// # 📌 Amac: TurkuazVM Engine composition root, config bootstrap ve local API process giris noktasini saglar
// # 📌 Modul - Rust
// # Version: 0.40.10
// # Aciklama: Controller, service, repository ve tool bagimliliklarini compose eder; request paniclerini API hatasina cevirerek Engine processinin sessizce dusmesini engeller
// # Bagimli Oldugu Katman: Controller | Service | Repo | Tool | View

mod config;
mod controllers;
mod services;
mod tools;
mod views;

use std::any::Any;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::thread;
use std::time::Instant;

use config::engine_config::EngineConfig;
use controllers::engine_api_controller::EngineApiController;
use services::engine_application_service::EngineApplicationService;
use turkuazvm_engine_api::EngineResponse;
use tools::engine_api_server_tool::EngineApiServerTool;
use views::console_view::ConsoleView;

fn main() {
    let config = match EngineConfig::load() {
        Ok(config) => config,
        Err(error) => {
            ConsoleView::render_engine_error(&format!("Config error: {error}"));
            return;
        }
    };

    let bind_ip = config.api_bind_ip;
    let bind_port = config.api_port;
    let api_timeout = config.api_timeout;
    let api_tls = config.api_tls.clone();
    let maintenance_interval = config.runtime_maintenance_interval;
    let service = EngineApplicationService::new(config);
    let mut controller = EngineApiController::new(service);
    let server = match EngineApiServerTool::bind(bind_ip, bind_port, api_timeout, api_tls.clone()) {
        Ok(server) => server,
        Err(error) => {
            ConsoleView::render_engine_error(&format!("Engine API bind failed: {error}"));
            return;
        }
    };

    let transport = if api_tls.is_some() { "tls" } else { "plain-loopback" };
    ConsoleView::render_engine_listening(&format!("{bind_ip}:{bind_port} [{transport}]"));

    loop {
        match server.try_accept() {
            Ok(Some(connection)) => {
                let request = connection.request.clone();
                let request_id = request.request_id;
                let started_at = Instant::now();
                let response = match catch_unwind(AssertUnwindSafe(|| controller.handle(request))) {
                    Ok(response) => response,
                    Err(payload) => {
                        let detail = panic_detail(payload.as_ref());
                        ConsoleView::render_engine_error(&format!(
                            "Engine request panic contained: request_id={request_id}, elapsed_ms={}, detail={detail}",
                            started_at.elapsed().as_millis()
                        ));
                        EngineResponse::failure(request_id, "engine_request_panic", detail)
                    }
                };
                if let Err(error) = EngineApiServerTool::respond(connection, &response) {
                    ConsoleView::render_engine_error(&format!(
                        "Engine API response failed: request_id={request_id}, elapsed_ms={}, error={error}",
                        started_at.elapsed().as_millis()
                    ));
                }
            }
            Ok(None) => {}
            Err(error) => {
                ConsoleView::render_engine_error(&format!("Engine API request failed: {error}"));
            }
        }
        if let Err(error) = controller.maintenance_tick() {
            ConsoleView::render_engine_error(&format!("Engine maintenance failed: {error}"));
        }
        thread::sleep(maintenance_interval);
    }
}

fn panic_detail(payload: &(dyn Any + Send)) -> String {
    if let Some(message) = payload.downcast_ref::<String>() {
        return message.clone();
    }
    if let Some(message) = payload.downcast_ref::<&str>() {
        return (*message).to_owned();
    }
    String::from("Rust panic payload has no string detail")
}
