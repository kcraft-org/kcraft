use slint::ComponentHandle;

pub fn spawn(app: &crate::AppWindow) -> std::thread::JoinHandle<()> {
    let weak = app.as_weak();
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async move {
            let mut rx = kcraft_net::NET_EVENTS.subscribe();
            while let Ok(event) = rx.recv().await {
                let weak = weak.clone();
                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(app) = weak.upgrade() {
                        app.set_progress_visible(true);
                        app.set_progress_name(event.job_name.into());
                        app.set_progress_completed(event.completed_actions as i32);
                        app.set_progress_total(event.total_actions as i32);
                    }
                });
            }
        });
    })
}
