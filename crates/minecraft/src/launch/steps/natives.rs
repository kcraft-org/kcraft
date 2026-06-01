use crate::launch::state::LogLevel;
use crate::launch::task::LaunchStep;
use crate::launch::task::LaunchTask;

pub struct ExtractNativesStep;

impl LaunchStep for ExtractNativesStep {
    fn execute(&mut self, task: &mut LaunchTask) -> Result<(), String> {
        let native_path = task.instance.get_native_path();
        std::fs::create_dir_all(&native_path)
            .map_err(|e| format!("Failed to create natives directory: {}", e))?;

        if let Ok(entries) = std::fs::read_dir(&native_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    let _ = std::fs::remove_file(&path);
                }
            }
        }

        let native_jars = task.instance.get_native_jars();
        for jar_path in &native_jars {
            let path = std::path::Path::new(jar_path);
            if !path.exists() {
                task.log(
                    &format!("Native library not found: {}", jar_path),
                    LogLevel::Warning,
                );
                continue;
            }

            task.log(
                &format!("Extracting natives from: {}", jar_path),
                LogLevel::Launcher,
            );

            let file = std::fs::File::open(path)
                .map_err(|e| format!("Failed to open {}: {}", jar_path, e))?;
            let mut archive = zip::ZipArchive::new(file)
                .map_err(|e| format!("Failed to read zip {}: {}", jar_path, e))?;

            for i in 0..archive.len() {
                let mut entry = archive
                    .by_index(i)
                    .map_err(|e| format!("Failed to read zip entry: {}", e))?;

                if entry.is_dir() {
                    continue;
                }

                let name = entry.name().to_string();
                if !name.ends_with(".so")
                    && !name.ends_with(".dll")
                    && !name.ends_with(".dylib")
                    && !name.ends_with(".jnilib")
                {
                    continue;
                }

                let out_path = std::path::Path::new(&native_path).join(
                    std::path::Path::new(&name)
                        .file_name()
                        .unwrap_or(std::ffi::OsStr::new(&name)),
                );
                let mut outfile = std::fs::File::create(&out_path)
                    .map_err(|e| format!("Failed to create {}: {}", out_path.display(), e))?;
                std::io::copy(&mut entry, &mut outfile)
                    .map_err(|e| format!("Failed to extract {}: {}", name, e))?;
            }
        }

        task.log(
            &format!("Natives extracted to: {}", native_path),
            LogLevel::Launcher,
        );
        Ok(())
    }

    fn abort(&mut self) -> bool {
        true
    }
    fn can_abort(&self) -> bool {
        false
    }
    fn finalize(&mut self) {}
    fn name(&self) -> &str {
        "ExtractNatives"
    }
}
