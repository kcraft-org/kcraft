use crate::data_root::data_root;
use crate::InstanceEntry;
use app_core::account::AccountType;
use auth::AccountList;
use minecraft::instance::Instance;
use minecraft::instance_list::InstanceList;
use minecraft::launch::{
    CheckJavaStep, CreateGameFoldersStep, DirectJavaLaunchStep, LaunchTask, PrintInstanceInfoStep,
};
use slint::{SharedString, VecModel};
use std::rc::Rc;
use tracing::info;

use slint::ComponentHandle;

pub fn load_model() -> Rc<VecModel<InstanceEntry>> {
    let inst_dir = data_root().join("instances");
    let inst_dir_str = inst_dir.to_string_lossy().to_string();
    let mut list = InstanceList::new(&inst_dir_str);
    let model = Rc::new(VecModel::<InstanceEntry>::from(vec![]));
    if let Err(e) = list.load_list() {
        tracing::warn!("Failed to load instances: {:?}", e);
        return model;
    }
    let mut vec = Vec::new();
    for i in 0..list.count() {
        if let Some(ptr) = list.at(i) {
            let inst = ptr.read();
            vec.push(InstanceEntry {
                id: inst.id().into(),
                name: inst.name.clone().into(),
                notes: inst.notes.clone().into(),
                java_path: inst.java_path.clone().into(),
                min_mem: inst.min_mem,
                max_mem: inst.max_mem,
                window_width: inst.window_width,
                window_height: inst.window_height,
                game_root: inst.game_root().into(),
                mods_root: inst.mods_root().into(),
            });
        }
    }
    model.set_vec(vec);
    model
}

pub fn setup_refresh(app: &crate::AppWindow) {
    let weak = app.as_weak();
    app.on_refresh_instances(move || {
        let app = match weak.upgrade() {
            Some(a) => a,
            None => return,
        };
        app.set_loading(true);
        app.set_error_message(SharedString::default());

        let new_instances = load_model();
        app.set_instances(new_instances.into());
        app.set_loading(false);
    });
}

pub fn setup_launch(app: &crate::AppWindow) {
    let weak = app.as_weak();
    app.on_launch_instance(move |id: SharedString| {
        let weak = weak.clone();
        let id = id.to_string();
        std::thread::spawn(move || {
            let inst_dir = data_root().join("instances");
            let inst_dir_str = inst_dir.to_string_lossy().to_string();
            let mut list = InstanceList::new(&inst_dir_str);
            if let Err(e) = list.load_list() {
                let _ = slint::invoke_from_event_loop({
                    let weak = weak.clone();
                    move || {
                        if let Some(app) = weak.upgrade() {
                            app.set_error_message(
                                format!("Failed to load instances: {}", e).into(),
                            );
                        }
                    }
                });
                return;
            }

            let ptr = match list.get_instance_by_id(&id) {
                Some(p) => p,
                None => {
                    let _ = slint::invoke_from_event_loop({
                        let weak = weak.clone();
                        let id = id.clone();
                        move || {
                            if let Some(app) = weak.upgrade() {
                                app.set_error_message(
                                    format!("Instance '{}' not found", id).into(),
                                );
                            }
                        }
                    });
                    return;
                }
            };

            let inst = ptr.read();
            let mut task = LaunchTask::new(Instance::new(&inst.instance_root, &inst.name));

            let java = if inst.java_path.is_empty() {
                java::find_java_paths()
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
                let mut session = minecraft::AuthSession::new(acc.data.profile_name());
                session.uuid = acc.data.profile_id().to_string();
                session.player_name = acc.data.profile_name().to_string();
                match acc.data.account_type {
                    AccountType::Offline => {
                        session.user_type = "legacy".to_string();
                        session.session = "offline".to_string();
                        session.access_token = "offline".to_string();
                    }
                    AccountType::Msa => {
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

            match task.execute() {
                Ok(_) => info!("Instance '{}' launched successfully", id),
                Err(e) => {
                    let _ = slint::invoke_from_event_loop(move || {
                        if let Some(app) = weak.upgrade() {
                            app.set_error_message(format!("Launch failed: {}", e).into());
                        }
                    });
                }
            }
        });
    });
}
