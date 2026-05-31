use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tracing::{Event, Subscriber};
use tracing_subscriber::layer::{Context, Layer, SubscriberExt};
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::Registry;

pub struct LogConfig {
    pub log_directory: PathBuf,
    pub max_log_files: u32,
    pub log_level: String,
}

impl Default for LogConfig {
    fn default() -> Self {
        let log_dir = {
            #[cfg(target_os = "linux")]
            {
                if let Ok(home) = std::env::var("HOME") {
                    PathBuf::from(home)
                        .join(".local")
                        .join("share")
                        .join("kcraft")
                        .join("logs")
                } else {
                    PathBuf::from("kcraft_logs")
                }
            }
            #[cfg(target_os = "windows")]
            {
                if let Ok(appdata) = std::env::var("APPDATA") {
                    PathBuf::from(appdata).join("kcraft").join("logs")
                } else {
                    PathBuf::from("kcraft_logs")
                }
            }
            #[cfg(target_os = "macos")]
            {
                if let Ok(home) = std::env::var("HOME") {
                    PathBuf::from(home)
                        .join("Library")
                        .join("Application Support")
                        .join("kcraft")
                        .join("logs")
                } else {
                    PathBuf::from("kcraft_logs")
                }
            }
            #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
            {
                PathBuf::from("kcraft_logs")
            }
        };
        LogConfig {
            log_directory: log_dir,
            max_log_files: 5,
            log_level: "debug".to_string(),
        }
    }
}

pub struct LogManager {
    config: LogConfig,
    file: Mutex<Option<File>>,
}

impl LogManager {
    pub fn new(config: LogConfig) -> Self {
        fs::create_dir_all(&config.log_directory).expect("Failed to create log directory");
        rotate_logs(&config.log_directory, config.max_log_files);
        let log_path = config.log_directory.join("KCraft-0.log");
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&log_path)
            .expect("Failed to open log file");
        LogManager {
            config,
            file: Mutex::new(Some(file)),
        }
    }

    pub fn init(self) {
        let level_filter: tracing_subscriber::filter::LevelFilter = self
            .config
            .log_level
            .parse()
            .unwrap_or(tracing_subscriber::filter::LevelFilter::DEBUG);

        let layer = LogLayer {
            manager: std::sync::Arc::new(self),
        };

        Registry::default().with(layer).with(level_filter).init();
    }
}

struct LogLayer {
    manager: std::sync::Arc<LogManager>,
}

impl<S: Subscriber> Layer<S> for LogLayer {
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        let metadata = event.metadata();
        let level = metadata.level();
        let target = metadata.target();
        let timestamp = chrono::Utc::now().format("%H:%M:%S%.3f").to_string();

        let mut message = String::new();
        let mut visitor = MessageVisitor(&mut message);
        event.record(&mut visitor);

        let level_char = match *level {
            tracing::Level::TRACE => 'T',
            tracing::Level::DEBUG => 'D',
            tracing::Level::INFO => 'I',
            tracing::Level::WARN => 'W',
            tracing::Level::ERROR => 'E',
        };

        let formatted = format!("{} {} [{}] {}\n", timestamp, level_char, target, message);

        eprint!("{}", formatted);

        if let Ok(mut file_guard) = self.manager.file.lock() {
            if let Some(ref mut file) = *file_guard {
                let _ = file.write_all(formatted.as_bytes());
                let _ = file.flush();
            }
        }
    }
}

struct MessageVisitor<'a>(&'a mut String);

impl<'a> tracing::field::Visit for MessageVisitor<'a> {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            self.0.push_str(&format!("{:?}", value));
        } else {
            self.0.push_str(&format!("{}={:?} ", field.name(), value));
        }
    }

    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        if field.name() == "message" {
            self.0.push_str(value);
        }
    }
}

fn rotate_logs(log_dir: &Path, max_files: u32) {
    let base = log_dir.join("KCraft");
    for i in (1..max_files).rev() {
        let src = format!("{}-{}.log", base.display(), i - 1);
        let dst = format!("{}-{}.log", base.display(), i);
        let src_path = Path::new(&src);
        let dst_path = Path::new(&dst);
        if src_path.exists() {
            let _ = fs::rename(src_path, dst_path);
        }
    }
}
