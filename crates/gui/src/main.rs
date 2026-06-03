#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod accounts;
mod data_root;
mod instances;
mod modpack;
mod net_events;

use data_root::data_root;
use tracing::info;

slint::include_modules!();

fn main() {
    let log_dir = data_root().join("logs");
    let config = logging::LogConfig {
        log_directory: log_dir,
        max_log_files: 5,
        log_level: "info".to_string(),
    };
    logging::LogManager::new(config).init();

    info!("Starting KCraft GUI (Slint)");

    let app = AppWindow::new().unwrap();

    app.set_accounts(accounts::load_model().into());
    app.set_instances(instances::load_model().into());

    instances::setup_refresh(&app);
    instances::setup_launch(&app);
    accounts::msa::setup_add_msa(&app);
    accounts::setup_add_offline(&app);
    accounts::setup_add_elyby(&app);
    modpack::setup(&app);

    let _net_handle = net_events::spawn(&app);

    app.run().unwrap();
}
