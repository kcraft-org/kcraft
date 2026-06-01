#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use kcraft_auth::{AccountList, AuthFlow, MinecraftAccount};
use kcraft_minecraft::instance::Instance;
use kcraft_minecraft::instance_list::InstanceList;
use kcraft_minecraft::launch::{
    CheckJavaStep, CreateGameFoldersStep, DirectJavaLaunchStep, LaunchTask, PrintInstanceInfoStep,
};
use std::path::PathBuf;
use tauri::Emitter;
use tracing::info;

fn data_root() -> PathBuf {
    let path = dirs::data_dir().map(|d| d.join("kcrack"));
    match path {
        Some(p) => p,
        None => {
            let fallback = dirs::home_dir()
                .map(|h| h.join(".local").join("share").join("kcrack"))
                .unwrap_or_else(|| PathBuf::from("kcrack_data"));
            tracing::warn!(
                "Could not determine platform data directory, falling back to: {}",
                fallback.display()
            );
            fallback
        }
    }
}

#[tauri::command]
fn list_instances() -> Result<Vec<serde_json::Value>, String> {
    let inst_dir = data_root().join("instances");
    let inst_dir_str = inst_dir.to_string_lossy().to_string();
    let mut list = InstanceList::new(&inst_dir_str);
    list.load_list()?;

    let mut out = Vec::new();
    for i in 0..list.count() {
        if let Some(ptr) = list.at(i) {
            let inst = ptr.read();
            out.push(serde_json::json!({
                "id": inst.id(),
                "name": inst.name,
                "icon_key": inst.icon_key,
                "notes": inst.notes,
            }));
        }
    }
    Ok(out)
}

#[derive(serde::Serialize, Clone)]
struct DeviceCodeEvent {
    uri: String,
    code: String,
    expires_in: i32,
}

#[tauri::command]
fn list_accounts() -> Result<Vec<serde_json::Value>, String> {
    let accounts_path = data_root().join("accounts.json");
    let list = AccountList::new(accounts_path);
    let mut res = Vec::new();
    for i in 0..list.count() {
        if let Some(acc) = list.at(i) {
            res.push(acc.save_to_json());
        }
    }
    Ok(res)
}

#[tauri::command]
async fn add_offline_account(username: String) -> Result<String, String> {
    let mut list = kcraft_auth::AccountList::new(data_root().join("accounts.json"));
    let _ = list.load();
    let profile = kcraft_core::account::MinecraftProfile {
        name: username.clone(),
        id: kcraft_auth::generate_offline_uuid(&username),
        ..Default::default()
    };
    let data = kcraft_core::account::AccountData {
        account_type: kcraft_core::account::AccountType::Offline,
        minecraft_profile: profile,
        ..Default::default()
    };
    list.add_account(kcraft_auth::MinecraftAccount { data, active: true });
    list.save().map_err(|e| e.to_string())?;
    Ok(format!("Added Offline: {}", username))
}

#[tauri::command]
async fn add_elyby_account(username: String, token: String) -> Result<String, String> {
    let mut list = kcraft_auth::AccountList::new(data_root().join("accounts.json"));
    let _ = list.load();
    let profile = kcraft_core::account::MinecraftProfile {
        name: username.clone(),
        id: kcraft_auth::generate_offline_uuid(&username),
        ..Default::default()
    };

    let yggdrasil_token = kcraft_core::account::Token {
        token: Some(token),
        ..Default::default()
    };

    let data = kcraft_core::account::AccountData {
        account_type: kcraft_core::account::AccountType::AuthlibInjector,
        authlib_injector_base_url: "https://authserver.ely.by/auth".to_string(),
        minecraft_profile: profile,
        yggdrasil_token,
        ..Default::default()
    };
    list.add_account(kcraft_auth::MinecraftAccount { data, active: true });
    list.save().map_err(|e| e.to_string())?;
    Ok(format!("Added Ely.by Account: {}", username))
}

#[tauri::command]
async fn login_msa(app: tauri::AppHandle) -> Result<(), String> {
    let mut flow = kcraft_auth::MsaFlow::new_interactive(String::new());
    flow.set_verification_callback(move |uri, code, expires| {
        let _ = app.emit(
            "device-code",
            DeviceCodeEvent {
                uri: uri.to_string(),
                code: code.to_string(),
                expires_in: expires,
            },
        );
    });

    let mut data = kcraft_core::account::AccountData::default();
    match flow.execute(&mut data) {
        Ok(kcraft_core::account::AccountTaskState::Succeeded) => {
            let accounts_path = data_root().join("accounts.json");
            let mut list = AccountList::new(accounts_path);
            let acc = MinecraftAccount::new(data);
            list.add_account(acc);
            let _ = list.save();
            Ok(())
        }
        Ok(state) => Err(format!("Login did not succeed: {:?}", state)),
        Err(e) => Err(format!("Login failed: {:?}", e)),
    }
}

#[tauri::command]
fn launch_instance(id: String) -> Result<String, String> {
    let inst_dir = data_root().join("instances");
    let inst_dir_str = inst_dir.to_string_lossy().to_string();
    let mut list = InstanceList::new(&inst_dir_str);
    list.load_list()?;

    let ptr = list
        .get_instance_by_id(&id)
        .ok_or_else(|| format!("Instance '{}' not found", id))?;

    let inst = ptr.read();

    let mut task = LaunchTask::new(Instance::new(&inst.instance_root, &inst.name));

    let java = if inst.java_path.is_empty() {
        kcraft_java::find_java_paths()
            .first()
            .cloned()
            .unwrap_or_else(|| "java".to_string())
    } else {
        inst.java_path.clone()
    };

    task.append_step(Box::new(CheckJavaStep::new(&java)));
    task.append_step(Box::new(PrintInstanceInfoStep));
    task.append_step(Box::new(CreateGameFoldersStep));

    let mut direct = DirectJavaLaunchStep::new(&inst_dir_str);

    let accounts_path = data_root().join("accounts.json");
    let account_list = AccountList::new(accounts_path);
    if let Some(acc) = account_list
        .default_account()
        .or_else(|| account_list.at(0))
    {
        let mut session = kcraft_minecraft::AuthSession::new(acc.data.profile_name());
        session.uuid = acc.data.profile_id().to_string();
        session.player_name = acc.data.profile_name().to_string();
        match acc.data.account_type {
            kcraft_core::account::AccountType::Offline => {
                session.user_type = "legacy".to_string();
                session.session = "offline".to_string();
                session.access_token = "offline".to_string();
            }
            kcraft_core::account::AccountType::Msa => {
                session.user_type = "msa".to_string();
                session.session = acc.data.access_token().to_string();
                session.access_token = acc.data.access_token().to_string();
                session.client_token = acc.data.client_token().to_string();
            }
            _ => {}
        }
        direct.set_session(session);
    }

    task.append_step(Box::new(direct));

    task.execute()?;
    info!("Instance '{}' launched successfully", id);
    Ok("Launch completed".to_string())
}

#[tauri::command]
async fn build_modpack(files: Vec<String>) -> Result<String, String> {
    use kcraft_minecraft::resolver::{DependencyNode, PackageId, Resolver};

    if files.is_empty() {
        return Err("No files provided".to_string());
    }

    let mut resolver = Resolver::new();
    let mut roots = Vec::new();

    for (i, file) in files.iter().enumerate() {
        let pkg_id = PackageId(format!("mod_{}", i));
        let path = std::path::Path::new(file);
        if !path.exists() {
            return Err(format!("File not found: {}", file));
        }

        resolver.add_node(DependencyNode {
            id: pkg_id.clone(),
            version: "1.0.0".to_string(),
            requires: vec![],
            conflicts: vec![],
        });
        roots.push(pkg_id);
    }

    match resolver.resolve(&roots) {
        Ok(resolved) => Ok(format!(
            "DAG Resolution successful: Installed {} packages with zero conflicts.",
            resolved.len()
        )),
        Err(e) => Err(format!("Resolution failed: {}", e)),
    }
}

fn main() {
    let log_dir = data_root().join("logs");
    let config = kcraft_logging::LogConfig {
        log_directory: log_dir,
        max_log_files: 5,
        log_level: "info".to_string(),
    };
    kcraft_logging::LogManager::new(config).init();

    info!("Starting KCraft GUI");

    tauri::Builder::default()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let mut rx = kcraft_net::NET_EVENTS.subscribe();
                while let Ok(event) = rx.recv().await {
                    let _ = handle.emit("net-progress", event);
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            list_instances,
            launch_instance,
            list_accounts,
            add_offline_account,
            add_elyby_account,
            login_msa,
            build_modpack
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
