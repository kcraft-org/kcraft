use crate::data_root::data_root;
use crate::AccountEntry;
use kcraft_auth::{AccountList, MinecraftAccount};
use kcraft_core::account::{AccountData, AccountType, MinecraftProfile};
use slint::{SharedString, VecModel};
use std::rc::Rc;

use slint::ComponentHandle;

pub fn load_model() -> Rc<VecModel<AccountEntry>> {
    let accounts_path = data_root().join("accounts.json");
    let list = AccountList::new(accounts_path);
    let model = Rc::new(VecModel::<AccountEntry>::from(vec![]));
    let mut vec = Vec::new();
    for i in 0..list.count() {
        if let Some(acc) = list.at(i) {
            let json = acc.save_to_json();
            let name = json
                .get("profile")
                .and_then(|p| p.get("name"))
                .and_then(|n| n.as_str())
                .unwrap_or("Unknown")
                .to_string();
            let type_name = match acc.data.account_type {
                AccountType::Offline => "Offline Account",
                AccountType::Msa => "Microsoft Account",
                AccountType::AuthlibInjector => "Ely.by Account",
            }
            .to_string();
            vec.push(AccountEntry {
                name: name.into(),
                type_name: type_name.into(),
                uuid: "".into(),
                token_validity: "".into(),
                token_status: "".into(),
            });
        }
    }
    model.set_vec(vec);
    model
}

fn refresh_accounts(app: &slint::Weak<crate::AppWindow>) {
    if let Some(app) = app.upgrade() {
        app.set_accounts(load_model().into());
    }
}

pub fn setup_add_offline(app: &crate::AppWindow) {
    let weak = app.as_weak();
    app.on_add_offline(move |username: SharedString| {
        if weak.upgrade().is_none() {
            return;
        }
        let mut list = AccountList::new(data_root().join("accounts.json"));
        let _ = list.load();
        let profile = MinecraftProfile {
            name: username.to_string(),
            id: kcraft_auth::generate_offline_uuid(username.as_ref()),
            ..Default::default()
        };
        let data = AccountData {
            account_type: AccountType::Offline,
            minecraft_profile: profile,
            ..Default::default()
        };
        list.add_account(MinecraftAccount { data, active: true });
        if let Err(e) = list.save() {
            tracing::error!("Failed to save account: {}", e);
        }
        refresh_accounts(&weak);
    });
}

pub fn setup_add_elyby(app: &crate::AppWindow) {
    let weak = app.as_weak();
    app.on_add_elyby(move |username: SharedString, token: SharedString| {
        if weak.upgrade().is_none() {
            return;
        }
        let mut list = AccountList::new(data_root().join("accounts.json"));
        let _ = list.load();
        let profile = MinecraftProfile {
            name: username.to_string(),
            id: kcraft_auth::generate_offline_uuid(username.as_ref()),
            ..Default::default()
        };
        let yggdrasil_token = kcraft_core::account::Token {
            token: Some(token.to_string()),
            ..Default::default()
        };
        let data = AccountData {
            account_type: AccountType::AuthlibInjector,
            authlib_injector_base_url: "https://authserver.ely.by/auth".to_string(),
            minecraft_profile: profile,
            yggdrasil_token,
            ..Default::default()
        };
        list.add_account(MinecraftAccount { data, active: true });
        if let Err(e) = list.save() {
            tracing::error!("Failed to save account: {}", e);
        }
        refresh_accounts(&weak);
    });
}
