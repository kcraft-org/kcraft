fn main() {
    let config =
        std::fs::read_to_string("tauri.conf.json").expect("Failed to read tauri.conf.json");
    if config.contains("UPDATE_PUBKEY_HERE") {
        println!("cargo:warning=tauri.conf.json: updater pubkey is still set to 'UPDATE_PUBKEY_HERE' - generate a real keypair before release");
    }
    tauri_build::build();
}
