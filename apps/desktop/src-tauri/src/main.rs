// # 📄 Dosya Yolu: /turkuazvm/apps/desktop/src-tauri/src/main.rs
// # 📌 Amac: TurkuazVM Desktop Tauri composition root ve runtime update bridge giris noktasini saglar
// # 📌 Modul - Rust
// # Version: 0.41.5
// # Aciklama: Kurulu Windows paketinde writable runtime/config kokunu LOCALAPPDATA altinda materialize eder; portable paket klasor-ici davranisini korur
// # Bagimli Oldugu Katman: Controller | Service | Tool | View

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config;
mod controllers;
mod services;
mod tools;
mod views;

use std::fs;
use std::path::Path;
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
const PORTABLE_MARKER_FILE: &str = "README-PORTABLE.txt";
const WINDOWS_LOCAL_APP_DATA_ENV: &str = "LOCALAPPDATA";
const RUNTIME_PRODUCT_DIRECTORY: &str = "TurkuazVM";
const RUNTIME_CONFIG_DIRECTORY: &str = "config";
const RUNTIME_DOWNLOAD_SOURCES_FILE: &str = "download-sources.yml";
const RUNTIME_STATIC_CONFIG_FILES: [&str; 2] = ["game-catalog.yml", "guest-catalog.yml"];

fn select_packaged_working_directory() -> Result<(), String> {
    if env::var_os(CONFIG_ENV).is_some() {
        return Ok(());
    }

    let executable_path = env::current_exe().map_err(|error| error.to_string())?;
    let executable_directory = executable_path
        .parent()
        .ok_or_else(|| String::from("Desktop executable directory could not be resolved"))?;
    if !executable_directory
        .join(PACKAGED_CONFIG_RELATIVE_PATH)
        .is_file()
    {
        return Ok(());
    }

    if executable_directory.join(PORTABLE_MARKER_FILE).is_file() {
        env::set_current_dir(executable_directory).map_err(|error| {
            format!("Portable runtime root could not be selected: {error}")
        })?;
        return Ok(());
    }

    #[cfg(windows)]
    {
        materialize_windows_runtime_config(executable_directory)?;
        return Ok(());
    }

    #[cfg(not(windows))]
    {
        env::set_current_dir(executable_directory).map_err(|error| {
            format!("Packaged runtime root could not be selected: {error}")
        })?;
        Ok(())
    }
}

#[cfg(windows)]
fn materialize_windows_runtime_config(install_root: &Path) -> Result<(), String> {
    let local_app_data = env::var_os(WINDOWS_LOCAL_APP_DATA_ENV)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| String::from("LOCALAPPDATA is not available for installed runtime"))?;
    let runtime_root = Path::new(&local_app_data).join(RUNTIME_PRODUCT_DIRECTORY);
    let runtime_config_root = runtime_root.join(RUNTIME_CONFIG_DIRECTORY);
    let install_config_root = install_root.join(RUNTIME_CONFIG_DIRECTORY);
    fs::create_dir_all(&runtime_config_root).map_err(|error| {
        format!(
            "Installed runtime config directory could not be created {}: {error}",
            runtime_config_root.display()
        )
    })?;
    fs::create_dir_all(runtime_root.join("data")).map_err(|error| {
        format!("Installed runtime data directory could not be created: {error}")
    })?;
    fs::create_dir_all(runtime_root.join("packages")).map_err(|error| {
        format!("Installed runtime package directory could not be created: {error}")
    })?;

    for file_name in RUNTIME_STATIC_CONFIG_FILES {
        copy_config_file(
            &install_config_root.join(file_name),
            &runtime_config_root.join(file_name),
            true,
        )?;
    }
    copy_config_file(
        &install_config_root.join(RUNTIME_DOWNLOAD_SOURCES_FILE),
        &runtime_config_root.join(RUNTIME_DOWNLOAD_SOURCES_FILE),
        false,
    )?;

    let packaged_config_path = install_config_root.join("turkuazvm.yml");
    let runtime_config_path = runtime_config_root.join("turkuazvm.yml");
    let packaged_config = fs::read_to_string(&packaged_config_path).map_err(|error| {
        format!(
            "Packaged runtime config could not be read {}: {error}",
            packaged_config_path.display()
        )
    })?;
    let runtime_config = materialize_runtime_paths(packaged_config, install_root)?;
    fs::write(&runtime_config_path, runtime_config).map_err(|error| {
        format!(
            "Installed runtime config could not be written {}: {error}",
            runtime_config_path.display()
        )
    })?;

    env::set_current_dir(&runtime_root).map_err(|error| {
        format!(
            "Installed runtime root could not be selected {}: {error}",
            runtime_root.display()
        )
    })?;
    Ok(())
}

#[cfg(windows)]
fn materialize_runtime_paths(mut content: String, install_root: &Path) -> Result<String, String> {
    let display_path = yaml_path(&install_root.join("bin/turkuazvm-display.exe"));
    let engine_path = yaml_path(&install_root.join("bin/turkuazvm-engine.exe"));
    let network_helper_path = yaml_path(&install_root.join("scripts/network_windows_managed.ps1"));
    let android_build_script_path = yaml_path(
        &install_root.join("guest/android-image/scripts/build_turkuaz_android_image.sh"),
    );

    content = replace_required(
        content,
        "display_executable_path: bin/turkuazvm-display.exe",
        &format!("display_executable_path: \"{display_path}\""),
    )?;
    content = replace_required(
        content,
        "executable_path: bin/turkuazvm-engine.exe",
        &format!("executable_path: \"{engine_path}\""),
    )?;
    content = replace_required(
        content,
        "managed_helper_path: scripts/network_windows_managed.ps1",
        &format!("managed_helper_path: \"{network_helper_path}\""),
    )?;
    content = replace_required(
        content,
        "build_script: guest/android-image/scripts/build_turkuaz_android_image.sh",
        &format!("build_script: \"{android_build_script_path}\""),
    )?;
    Ok(content)
}

#[cfg(windows)]
fn copy_config_file(source: &Path, destination: &Path, overwrite: bool) -> Result<(), String> {
    if !overwrite && destination.is_file() {
        return Ok(());
    }
    if !source.is_file() {
        return Err(format!("Packaged config asset not found: {}", source.display()));
    }
    fs::copy(source, destination).map(|_| ()).map_err(|error| {
        format!(
            "Config asset copy failed {} -> {}: {error}",
            source.display(),
            destination.display()
        )
    })
}

#[cfg(windows)]
fn replace_required(content: String, from: &str, to: &str) -> Result<String, String> {
    if !content.contains(from) {
        return Err(format!("Packaged config token not found: {from}"));
    }
    Ok(content.replacen(from, to, 1))
}

#[cfg(windows)]
fn yaml_path(path: &Path) -> String {
    path.to_string_lossy()
        .replace('\\', "/")
        .replace('"', "\\\"")
}

fn main() {
    if let Err(error) = select_packaged_working_directory() {
        ConsoleView::render_error(&format!("Runtime root error: {error}"));
        return;
    }

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
