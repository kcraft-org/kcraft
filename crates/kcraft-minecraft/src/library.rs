use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::gradle_specifier::GradleSpecifier;
use crate::mojang_download_info::MojangLibraryDownloadInfo;
use crate::rule::{Rule, RuleAction};
use crate::OpSys;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Library {
    pub name: GradleSpecifier,
    pub repository_url: Option<String>,
    pub absolute_url: Option<String>,
    pub filename: Option<String>,
    pub display_name: Option<String>,
    pub hint: Option<String>,
    pub storage_prefix: Option<String>,
    pub has_excludes: bool,
    pub extract_excludes: Vec<String>,
    pub native_classifiers: HashMap<String, String>,
    pub rules: Vec<Rule>,
    pub mojang_downloads: Option<MojangLibraryDownloadInfo>,
    pub apply_rules: bool,
}

impl Library {
    pub fn new(name: impl Into<GradleSpecifier>) -> Self {
        Library {
            name: name.into(),
            repository_url: None,
            absolute_url: None,
            filename: None,
            display_name: None,
            hint: None,
            storage_prefix: None,
            has_excludes: false,
            extract_excludes: Vec::new(),
            native_classifiers: HashMap::new(),
            rules: Vec::new(),
            mojang_downloads: None,
            apply_rules: false,
        }
    }

    pub fn raw_name(&self) -> &GradleSpecifier {
        &self.name
    }

    pub fn artifact_prefix(&self) -> String {
        self.name.artifact_prefix()
    }

    pub fn artifact_id(&self) -> &str {
        self.name.artifact_id()
    }

    pub fn version(&self) -> &str {
        self.name.version()
    }

    pub fn is_native(&self) -> bool {
        !self.native_classifiers.is_empty()
    }

    pub fn is_active(&self, os: &OpSys) -> bool {
        if self.rules.is_empty() {
            return true;
        }
        for rule in &self.rules {
            match rule.apply(os) {
                RuleAction::Allow => return true,
                RuleAction::Disallow => return false,
                RuleAction::Defer => continue,
            }
        }
        true
    }

    pub fn filename_for(&self, os: &OpSys) -> String {
        if let Some(ref fname) = self.filename {
            return fname.clone();
        }

        let mut spec = self.name.clone();
        if let Some(native) = self.get_compatible_native(os) {
            spec.set_classifier(native);
        }
        spec.get_file_name()
    }

    pub fn display_name_for(&self, os: &OpSys) -> String {
        self.filename_for(os)
    }

    pub fn get_compatible_native(&self, os: &OpSys) -> Option<String> {
        let classifier = os.classifier();
        if let Some(native) = self.native_classifiers.get(classifier) {
            return Some(native.clone());
        }
        // Try architecture-specific classifiers
        let arch = std::env::consts::ARCH;
        let arch_specific = match arch {
            "x86_64" => format!("natives-{}", classifier),
            "aarch64" => format!("natives-{}-arm64", classifier),
            _ => format!("natives-{}", classifier),
        };
        Some(arch_specific)
    }

    pub fn is_local(&self) -> bool {
        false
    }

    pub fn is_always_stale(&self) -> bool {
        false
    }

    pub fn is_forge(&self) -> bool {
        self.hint.as_deref() == Some("forge")
    }

    pub fn local_path(&self) -> String {
        self.name.to_path(None)
    }

    pub fn download_info(&self) -> Option<&crate::mojang_download_info::MojangDownloadInfo> {
        self.mojang_downloads.as_ref().and_then(|dls| dls.artifact.as_ref())
    }

    pub fn storage_prefix(&self) -> String {
        self.storage_prefix.clone().unwrap_or_else(default_storage_prefix)
    }

    pub fn storage_suffix(&self, os: &OpSys) -> String {
        let filename = self.filename_for(os);
        self.name.to_path(Some(&filename))
    }

    pub fn get_applicable_files(
        &self,
        os: &OpSys,
    ) -> (Vec<String>, Vec<String>, Vec<String>, Vec<String>) {
        let mut jar = Vec::new();
        let mut native = Vec::new();
        let mut native32 = Vec::new();
        let mut native64 = Vec::new();

        if !self.is_active(os) {
            return (jar, native, native32, native64);
        }

        if self.is_native() {
            let compat = self.get_compatible_native(os);
            let prefix = self.storage_prefix();
            let suffix = self.name.to_path(compat.as_ref().map(|s| {
                let mut spec = self.name.clone();
                spec.set_classifier(s.clone());
                spec.get_file_name()
            }).as_deref());
            let path = format!("{}/{}", prefix, suffix);

            // Check if it's 32-bit or 64-bit
            if let Some(nc) = self.native_classifiers.get(os.classifier()) {
                if nc.contains("32") || nc.contains("86") {
                    native32.push(path.clone());
                } else {
                    native64.push(path.clone());
                }
            }
            native.push(path);
        } else {
            let prefix = self.storage_prefix();
            let suffix = self.storage_suffix(os);
            let path = format!("{}/{}", prefix, suffix);
            jar.push(path);
        }

        (jar, native, native32, native64)
    }
}

fn default_storage_prefix() -> String {
    "libraries".to_string()
}
