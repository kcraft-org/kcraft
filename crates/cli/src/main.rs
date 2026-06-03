use std::path::{Path, PathBuf};
use std::process::exit;

use auth::account_list::{AccountList, MinecraftAccount};
use config::settings_object::{IniSettingsStorage, SettingsObject};
use app_core::build_config::BUILD_CONFIG;
use http_cache::HttpMetaCache;
use minecraft::instance_list::InstanceList;
use minecraft::launch::*;
use minecraft::MinecraftServerTarget;
use tracing::info;

fn determine_data_path() -> PathBuf {
    let data_dir = {
        #[cfg(target_os = "linux")]
        {
            if let Ok(home) = std::env::var("HOME") {
                PathBuf::from(home)
                    .join(".local")
                    .join("share")
                    .join("kcraft")
            } else {
                PathBuf::from("kcraft_data")
            }
        }
        #[cfg(target_os = "windows")]
        {
            if let Ok(appdata) = std::env::var("APPDATA") {
                PathBuf::from(appdata).join("kcraft")
            } else {
                PathBuf::from("kcraft_data")
            }
        }
        #[cfg(target_os = "macos")]
        {
            if let Ok(home) = std::env::var("HOME") {
                PathBuf::from(home)
                    .join("Library")
                    .join("Application Support")
                    .join("kcraft")
            } else {
                PathBuf::from("kcraft_data")
            }
        }
        #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
        {
            PathBuf::from("kcraft_data")
        }
    };
    let _ = std::fs::create_dir_all(&data_dir);
    data_dir
}

fn init_logging(data_path: &Path) {
    let log_dir = data_path.join("logs");
    let _ = std::fs::create_dir_all(&log_dir);

    let config = logging::LogConfig {
        log_directory: log_dir,
        max_log_files: 5,
        log_level: "debug".to_string(),
    };

    let log_manager = logging::LogManager::new(config);
    log_manager.init();
    info!("Logger initialized");
}

fn init_settings(data_path: &Path) -> SettingsObject {
    let config_path = data_path.join(&BUILD_CONFIG.launcher_config_file);
    let storage = IniSettingsStorage::new(config_path);
    let settings = SettingsObject::new(Box::new(storage)).expect("Failed to initialize settings");

    settings.register_simple(
        "InstanceDir",
        data_path.join("instances").to_string_lossy().to_string(),
    );
    settings.register_simple("JavaPath", String::new());
    settings.register_simple("JavaMemory", 2048i32);
    settings.register_simple("Theme", "dark".to_string());
    settings.register_simple("AutoUpdate", true);

    info!("Settings initialized");
    settings
}

fn init_meta_cache(data_path: &Path) -> HttpMetaCache {
    let cache_path = data_path.join("cache").join("index.json");
    let _ = std::fs::create_dir_all(data_path.join("cache"));

    let cache = HttpMetaCache::new(cache_path);
    cache.add_base("meta", data_path.join("cache").join("meta"));
    cache.add_base("assets", data_path.join("cache").join("assets"));
    cache.add_base("libraries", data_path.join("cache").join("libraries"));
    cache.add_base("skins", data_path.join("cache").join("skins"));

    info!("Meta cache initialized");
    cache
}

fn init_accounts(data_path: &Path) -> AccountList {
    let accounts_path = data_path.join("accounts.json");
    let mut list = AccountList::new(accounts_path.clone());
    list.set_autosave(true);

    if list.count() == 0 {
        info!("No accounts found. Creating default offline account.");
        let offline_account = MinecraftAccount::create_offline("Player");
        list.add_account(offline_account);
    }

    info!("Accounts initialized: {} account(s)", list.count());
    list
}

fn print_help() {
    println!("KCraft - Custom Minecraft Launcher");
    println!("Usage: kcraft [options]");
    println!("Options:");
    println!("  --list                 List all instances");
    println!("  --launch <instance>    Launch an instance by ID");
    println!("  --server <host:port>   Server to join after launch");
    println!("  --profile <name>       Account profile to use");
    println!("  --dir <path>           Data directory path");
    println!("  --help                 Show this help");
}

fn parse_cli_args() -> (
    Option<String>,
    Option<String>,
    Option<String>,
    bool,
    Option<PathBuf>,
) {
    let args: Vec<String> = std::env::args().collect();
    let mut launch = None;
    let mut server = None;
    let mut profile = None;
    let mut list = false;
    let mut dir = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--launch" if i + 1 < args.len() => {
                launch = Some(args[i + 1].clone());
                i += 1;
            }
            "--server" if i + 1 < args.len() => {
                server = Some(args[i + 1].clone());
                i += 1;
            }
            "--profile" if i + 1 < args.len() => {
                profile = Some(args[i + 1].clone());
                i += 1;
            }
            "--list" => {
                list = true;
            }
            "--dir" if i + 1 < args.len() => {
                dir = Some(PathBuf::from(&args[i + 1]));
                i += 1;
            }
            "--help" | "-h" => {
                print_help();
                exit(0);
            }
            _ => {}
        }
        i += 1;
    }

    (launch, server, profile, list, dir)
}

fn print_instance_list(instances: &InstanceList) {
    println!();
    println!("Instances:");
    println!("{:-<80}", "");
    println!("  {:<4} {:<30} {:<30} Status", "Idx", "Name", "ID");
    println!("{:-<80}", "");
    for i in 0..instances.count() {
        if let Some(inst_ptr) = instances.at(i) {
            let inst = inst_ptr.read();
            let status = if inst.has_broken_version {
                "BROKEN"
            } else if inst.crashed {
                "CRASHED"
            } else {
                "OK"
            };
            println!(
                "  [{:>3}] {:<30} {:<30} {}",
                i,
                inst.name,
                inst.id(),
                status
            );
        }
    }
    println!("{:-<80}", "");
    println!("  Total: {} instance(s)", instances.count());
    println!();
}

fn build_launch_task(
    instance: &minecraft::Instance,
    session: Option<minecraft::AuthSession>,
    server: Option<MinecraftServerTarget>,
    data_root: &str,
) -> LaunchTask {
    let mut task = LaunchTask::new(instance.clone());
    task.session = session;
    task.server = server;

    task.append_step(Box::new(PrintInstanceInfoStep));
    task.append_step(Box::new(CreateGameFoldersStep));
    task.append_step(Box::new(VerifyJavaInstallStep));
    task.append_step(Box::new(CheckJavaStep::new(&instance.java_path)));
    task.append_step(Box::new(ExtractNativesStep));
    task.append_step(Box::new(ReconstructAssetsStep));
    task.append_step(Box::new(ScanModFoldersStep));
    task.append_step(Box::new(ClaimAccountStep));

    let mut launch_step = DirectJavaLaunchStep::new(&instance.game_root());
    launch_step.set_lib_dir(data_root);
    if let Some(ref sess) = task.session {
        launch_step.set_session(sess.clone());
    }
    if let Some(ref srv) = task.server {
        launch_step.set_server(srv.clone());
    }
    task.append_step(Box::new(launch_step));

    task
}

#[tokio::main]
async fn main() {
    let (launch_id, server_addr, profile_name, list_flag, custom_dir) = parse_cli_args();

    let data_path = custom_dir.unwrap_or_else(determine_data_path);
    init_logging(&data_path);

    info!(
        "{} v{}",
        BUILD_CONFIG.launcher_display_name,
        BUILD_CONFIG.version_string()
    );
    info!(
        "Platform: {} {}",
        BUILD_CONFIG.build_platform,
        std::env::consts::ARCH
    );
    info!("Data path: {}", data_path.display());

    let _settings = init_settings(&data_path);
    let _cache = init_meta_cache(&data_path);
    let accounts = init_accounts(&data_path);

    let default_account = accounts.default_account().cloned();
    if let Some(ref account) = default_account {
        info!(
            "Default account: {} ({})",
            account.data.profile_name(),
            account.data.account_type.as_str()
        );
    }

    let inst_dir = _settings
        .get::<String>("InstanceDir")
        .unwrap_or_else(|_| data_path.join("instances").to_string_lossy().to_string());

    if list_flag {
        let mut instance_list = InstanceList::new(&inst_dir);
        match instance_list.load_list() {
            Ok(()) => print_instance_list(&instance_list),
            Err(e) => {
                eprintln!("Error loading instances: {}", e);
                exit(1);
            }
        }
        return;
    }

    if let Some(instance_id) = launch_id {
        let mut instance_list = InstanceList::new(&inst_dir);
        if let Err(e) = instance_list.load_list() {
            eprintln!("Error loading instances: {}", e);
            exit(1);
        }

        let instance_ptr = match instance_list.get_instance_by_id(&instance_id) {
            Some(p) => p,
            None => {
                eprintln!("Instance '{}' not found", instance_id);
                info!("Available instances:");
                for i in 0..instance_list.count() {
                    if let Some(inst_ptr) = instance_list.at(i) {
                        let inst = inst_ptr.read();
                        info!("  {} ({})", inst.name, inst.id());
                    }
                }
                exit(1);
            }
        };

        let inst = instance_ptr.write();

        if !inst.can_launch() {
            eprintln!(
                "Instance '{}' has broken version and cannot be launched",
                instance_id
            );
            exit(1);
        }

        let account = if let Some(ref name) = profile_name {
            accounts.get_account_by_profile_name(name).cloned()
        } else {
            accounts.default_account().cloned()
        };

        let session = account.map(|a| minecraft::AuthSession::new(a.data.profile_name()));

        let server = server_addr.map(|addr| MinecraftServerTarget::parse(&addr));

        info!("Launching instance: {} ({})", inst.name, instance_id);

        let mut task = build_launch_task(&inst, session, server, &data_path.to_string_lossy());
        match task.execute() {
            Ok(()) => info!("Minecraft exited cleanly"),
            Err(e) => {
                eprintln!("Launch failed: {}", e);
                exit(1);
            }
        }
    } else {
        print_help();
    }
}
