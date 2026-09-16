// # 📄 Dosya Yolu: /turkuazvm/apps/desktop/src-tauri/src/main.rs
// # 📌 Amac: TurkuazVM Desktop Tauri composition root ve runtime update bridge giris noktasini saglar
// # 📌 Modul - Rust
// # Version: 0.41.4
// # Aciklama: Paketli runtime kokunu EXE yanindaki config ile sabitler; Host profilleri, Engine API, VM lifecycle, ag medya, Connection Center ve Android Runtime commandlarini compose eder
// # Bagimli Oldugu Katman: Controller | Service | Tool | View

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config;
mod controllers;
mod services;
mod tools;
mod views;

use std::{env, thread};

use tauri::{Emitter, Manager};

use config::desktop_config::DesktopConfig;
use controllers::desktop_controller::{
    apply_android_display, assign_android_image, attach_default_network, attach_network_profile, publish_vm_service, unpublish_vm_service, prepare_vm_ssh_access, prepare_vm_rdp_access, test_tcp_connection, open_ssh_connection, open_rdp_connection, clone_vm, configure_android_runtime, create_snapshot, create_vm, create_vm_disk,
    define_android_image, delete_snapshot, get_android_image_assignment, get_android_profile, get_android_status,
    get_dashboard, get_gpu_capabilities, get_network_overview, get_storage_overview, get_download_settings, save_download_settings,
    get_artifact_cache_overview, list_artifact_cache_entries, set_artifact_cache_pinned,
    remove_artifact_cache_entry, verify_artifact_cache, cleanup_artifact_cache,
    revalidate_artifact_cache, revalidate_all_artifact_cache, fetch_mutable_artifact_cache, list_android_images, prepare_android_image_build, register_android_image_build,
    install_android_image_distribution, cancel_android_image_distribution, cleanup_android_image_distribution,
    open_android_image_install_log,
    apply_game_profile, configure_gaming_input_profile, detect_games, get_game_compatibility,
    get_gaming_input_capabilities, get_gaming_input_profile, get_guest_agent_status,
    inject_android_input, list_game_catalog, list_guest_catalog, reset_gaming_input_state,
    install_android_apk, launch_android_package, list_android_packages, list_hosts, list_snapshots,
    list_vms, open_display, restore_snapshot, select_host, start_vm, stop_android_package, stop_vm, detach_vm_network,
    resize_vm_disk, delete_vm_disk, update_vm, delete_vm, configure_installer_media, eject_installer_media,
    start_installer_media_download, get_installer_media_download, cancel_installer_media_download, attach_downloaded_installer_media, open_external_url,
    pick_installer_iso, list_local_logs, open_local_log,
    uninstall_android_package, wait_android_ready,
};
use services::app_state::{DesktopAppState, EVENT_RUNTIME_UPDATE};
use services::desktop_service::DesktopService;
use views::console_view::ConsoleView;

const CONFIG_ENV: &str = "TURKUAZVM_CONFIG";
const PACKAGED_CONFIG_RELATIVE_PATH: &str = "config/turkuazvm.yml";

fn select_packaged_working_directory() {
    if env::var_os(CONFIG_ENV).is_some() {
        return;
    }

    let Ok(executable_path) = env::current_exe() else {
        return;
    };
    let Some(executable_directory) = executable_path.parent() else {
        return;
    };
    if executable_directory
        .join(PACKAGED_CONFIG_RELATIVE_PATH)
        .is_file()
    {
        let _ = env::set_current_dir(executable_directory);
    }
}

fn main() {
    select_packaged_working_directory();

    let config = match DesktopConfig::load() {
        Ok(config) => config,
        Err(error) => {
            ConsoleView::render_error(&format!("Config error: {error}"));
            return;
        }
    };

    let refresh_interval = config.refresh_interval;
    let state = DesktopAppState::new(DesktopService::new(config.clone()));

    tauri::Builder::default()
        .setup(move |app| {
            app.manage(state);
            let app_handle = app.handle().clone();
            thread::spawn(move || loop {
                let managed_state = app_handle.state::<DesktopAppState>();
                if let Ok(mut service) = managed_state.service.lock() {
                    if let Ok(dashboard) = service.dashboard() {
                        let _ = app_handle.emit(EVENT_RUNTIME_UPDATE, dashboard);
                    }
                }
                thread::sleep(refresh_interval);
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_dashboard,
            get_gpu_capabilities,
            get_storage_overview,
            get_download_settings,
            save_download_settings,
            get_artifact_cache_overview,
            list_artifact_cache_entries,
            set_artifact_cache_pinned,
            remove_artifact_cache_entry,
            verify_artifact_cache,
            cleanup_artifact_cache,
            revalidate_artifact_cache,
            revalidate_all_artifact_cache,
            fetch_mutable_artifact_cache,
            create_vm_disk,
            resize_vm_disk,
            delete_vm_disk,
            update_vm,
            delete_vm,
            configure_installer_media,
            eject_installer_media,
            start_installer_media_download,
            get_installer_media_download,
            cancel_installer_media_download,
            attach_downloaded_installer_media,
            open_external_url,
            pick_installer_iso,
            list_local_logs,
            open_local_log,
            get_network_overview,
            attach_default_network,
            attach_network_profile,
            publish_vm_service,
            unpublish_vm_service,
            prepare_vm_ssh_access,
            prepare_vm_rdp_access,
            test_tcp_connection,
            open_ssh_connection,
            open_rdp_connection,
            detach_vm_network,
            list_hosts,
            select_host,
            list_vms,
            list_guest_catalog,
            create_vm,
            start_vm,
            stop_vm,
            open_display,
            create_snapshot,
            list_snapshots,
            restore_snapshot,
            delete_snapshot,
            clone_vm,
            list_android_images,
            define_android_image,
            prepare_android_image_build,
            register_android_image_build,
            install_android_image_distribution,
            cancel_android_image_distribution,
            cleanup_android_image_distribution,
            open_android_image_install_log,
            assign_android_image,
            get_android_image_assignment,
            configure_android_runtime,
            get_android_profile,
            get_android_status,
            wait_android_ready,
            apply_android_display,
            list_android_packages,
            install_android_apk,
            uninstall_android_package,
            launch_android_package,
            stop_android_package,
            inject_android_input,
            get_gaming_input_capabilities,
            get_gaming_input_profile,
            configure_gaming_input_profile,
            reset_gaming_input_state,
            get_guest_agent_status,
            list_game_catalog,
            detect_games,
            get_game_compatibility,
            apply_game_profile
        ])
        .run(tauri::generate_context!())
        .expect("TurkuazVM Desktop runtime failed");
}
