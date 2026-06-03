use crate::accounts;
use crate::data_root::data_root;
use auth::{AuthFlow, MinecraftAccount};
use app_core::account::{AccountData, AccountTaskState};

use slint::ComponentHandle;

pub fn setup_add_msa(app: &crate::AppWindow) {
    let weak = app.as_weak();
    app.on_add_msa(move || {
        if let Some(app) = weak.upgrade() {
            app.set_msa_is_visible(true);
        }

        let weak = weak.clone();
        std::thread::spawn(move || {
            let mut flow = auth::MsaFlow::new_interactive(String::new());

            let weak2 = weak.clone();
            flow.set_verification_callback(move |uri, code, _expires| {
                let weak3 = weak2.clone();
                let uri = uri.to_string();
                let code = code.to_string();
                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(app) = weak3.upgrade() {
                        app.set_msa_uri(uri.into());
                        app.set_msa_code(code.into());
                    }
                });
            });

            let mut data = AccountData::default();
            let result =
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| flow.execute(&mut data)));

            let result = match result {
                Ok(r) => r,
                Err(_) => {
                    let _ = slint::invoke_from_event_loop(move || {
                        if let Some(app) = weak.upgrade() {
                            app.set_error_message(
                                "MSA login encountered an error. Please try again.".into(),
                            );
                            app.set_msa_is_visible(false);
                        }
                    });
                    return;
                }
            };

            let _ = slint::invoke_from_event_loop(move || {
                if let Some(app) = weak.upgrade() {
                    match result {
                        Ok(AccountTaskState::Succeeded) => {
                            let accounts_path = data_root().join("accounts.json");
                            let mut list = auth::AccountList::new(accounts_path);
                            let acc = MinecraftAccount::new(data);
                            list.add_account(acc);
                            let _ = list.save();
                            app.set_accounts(accounts::load_model().into());
                        }
                        Ok(state) => {
                            tracing::warn!("Login did not succeed: {:?}", state);
                        }
                        Err(e) => {
                            tracing::warn!("Login failed: {:?}", e);
                        }
                    }
                    app.set_msa_is_visible(false);
                }
            });
        });
    });
}
