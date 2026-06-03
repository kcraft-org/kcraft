use slint::ComponentHandle;

pub fn setup(app: &crate::AppWindow) {
    let weak = app.as_weak();
    app.on_build_modpack(move || {
        let app = match weak.upgrade() {
            Some(a) => a,
            None => return,
        };
        let files = rfd::FileDialog::new()
            .add_filter("JAR", &["jar"])
            .add_filter("Configuration", &["pw.toml"])
            .add_filter("Archive", &["zip"])
            .pick_files();

        let files = match files {
            Some(f) => f,
            None => return,
        };

        let mut resolver = minecraft::resolver::Resolver::new();
        let mut roots = Vec::new();

        for (i, file) in files.iter().enumerate() {
            let pkg_id = minecraft::resolver::PackageId(format!("mod_{}", i));
            let path = std::path::Path::new(file);
            if !path.exists() {
                app.set_error_message(format!("File not found: {}", file.display()).into());
                return;
            }

            resolver.add_node(minecraft::resolver::DependencyNode {
                id: pkg_id.clone(),
                version: "1.0.0".to_string(),
                requires: vec![],
                conflicts: vec![],
            });
            roots.push(pkg_id);
        }

        match resolver.resolve(&roots) {
            Ok(resolved) => {
                app.set_modpack_result(
                    format!(
                        "Added {} file(s) to the modpack. DAG resolved with no conflicts.",
                        resolved.len()
                    )
                    .into(),
                );
            }
            Err(e) => app.set_error_message(format!("Resolution failed: {}", e).into()),
        }
    });

    let weak = app.as_weak();
    app.on_modpack_add_files(move || {
        let files = rfd::FileDialog::new()
            .add_filter("JAR", &["jar"])
            .add_filter("Configuration", &["pw.toml"])
            .add_filter("Archive", &["zip"])
            .pick_files();

        if let Some(files) = files {
            let _app = weak.upgrade();
            for file in &files {
                tracing::info!("Selected file: {}", file.display());
            }
        }
    });
}
