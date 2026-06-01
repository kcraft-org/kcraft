use chrono::{DateTime, TimeZone, Utc};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct FMLlib {
    pub filename: String,
    pub checksum: String,
}

#[derive(Debug, Clone)]
pub struct VersionFilterData {
    pub fml_libs_mapping: HashMap<String, Vec<FMLlib>>,
    pub forge_installer_blacklist: HashSet<String>,
    pub legacy_cutoff_date: DateTime<Utc>,
    pub lwjgl_whitelist: HashSet<String>,
    pub java8_begins_date: DateTime<Utc>,
    pub java16_begins_date: DateTime<Utc>,
    pub java17_begins_date: DateTime<Utc>,
}

impl VersionFilterData {
    pub fn new() -> Self {
        let make_date = |y, m, d| {
            Utc.with_ymd_and_hms(y, m, d, 0, 0, 0)
                .single()
                .expect("invalid date")
        };

        let mut fml_libs = HashMap::new();
        fml_libs.insert(
            "1.7.10".to_string(),
            vec![
                FMLlib {
                    filename: "scala-library-2.1.0.jar".to_string(),
                    checksum: "32fa1396e31d1eb322378aeb675bc1571507c3ea".to_string(),
                },
                FMLlib {
                    filename: "scala-compiler-2.1.0.jar".to_string(),
                    checksum: "a585bfaabe95fa1e26c178b255fa62ed63a22d45".to_string(),
                },
            ],
        );

        let mut lwjgl = HashSet::new();
        lwjgl.insert("org.lwjgl.lwjgl:lwjgl".to_string());
        lwjgl.insert("org.lwjgl.lwjgl:lwjgl_util".to_string());
        lwjgl.insert("org.lwjgl.lwjgl:lwjgl-platform".to_string());
        lwjgl.insert("net.java.jinput:jinput".to_string());
        lwjgl.insert("net.java.jutils:jutils".to_string());

        let mut forge_blacklist = HashSet::new();
        forge_blacklist.insert("1.7.10".to_string());
        forge_blacklist.insert("1.8.9".to_string());
        forge_blacklist.insert("1.9".to_string());
        forge_blacklist.insert("1.9.4".to_string());
        forge_blacklist.insert("1.10".to_string());
        forge_blacklist.insert("1.10.2".to_string());
        forge_blacklist.insert("1.11".to_string());
        forge_blacklist.insert("1.11.2".to_string());
        forge_blacklist.insert("1.12".to_string());
        forge_blacklist.insert("1.12.1".to_string());
        forge_blacklist.insert("1.12.2".to_string());

        VersionFilterData {
            fml_libs_mapping: fml_libs,
            forge_installer_blacklist: forge_blacklist,
            legacy_cutoff_date: make_date(2014, 1, 1),
            lwjgl_whitelist: lwjgl,
            java8_begins_date: make_date(2017, 3, 30), // 17w13a
            java16_begins_date: make_date(2021, 5, 12), // 21w19a
            java17_begins_date: make_date(2021, 9, 15), // 1.18 Pre Release 2
        }
    }

    pub fn get_java_major_for_version(&self, release_date: &DateTime<Utc>) -> i32 {
        if *release_date >= self.java17_begins_date {
            17
        } else if *release_date >= self.java16_begins_date {
            16
        } else if *release_date >= self.java8_begins_date {
            8
        } else {
            6
        }
    }
}

impl Default for VersionFilterData {
    fn default() -> Self {
        Self::new()
    }
}

pub static G_VERSION_FILTER_DATA: once_cell::sync::Lazy<VersionFilterData> =
    once_cell::sync::Lazy::new(VersionFilterData::new);
