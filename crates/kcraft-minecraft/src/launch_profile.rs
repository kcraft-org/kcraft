use serde::{Deserialize, Serialize};

use crate::library::Library;
use crate::version_file::VersionFile;
use crate::OpSys;
use crate::RequireSet;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaunchProfile {
    pub minecraft_version: Option<String>,
    pub main_class: Option<String>,
    pub applet_class: Option<String>,
    pub minecraft_arguments: Option<String>,
    pub additional_jvm_arguments: Vec<String>,
    pub compatible_java_majors: Vec<i32>,
    pub libraries: Vec<Library>,
    pub native_libraries: Vec<Library>,
    pub main_jar: Option<Library>,
    pub minecraft_assets: Option<String>,
    pub assets: String,
    pub traits: Vec<String>,
    pub add_tweakers: Vec<String>,
    pub agents: Vec<crate::agent::Agent>,
    pub type_: Option<String>,
    pub required: RequireSet,
    pub conflicts: RequireSet,
}

impl LaunchProfile {
    pub fn new() -> Self {
        LaunchProfile {
            minecraft_version: None,
            main_class: None,
            applet_class: None,
            minecraft_arguments: None,
            additional_jvm_arguments: Vec::new(),
            compatible_java_majors: Vec::new(),
            libraries: Vec::new(),
            native_libraries: Vec::new(),
            main_jar: None,
            minecraft_assets: None,
            assets: "legacy".to_string(),
            traits: Vec::new(),
            add_tweakers: Vec::new(),
            agents: Vec::new(),
            type_: None,
            required: Vec::new(),
            conflicts: Vec::new(),
        }
    }

    pub fn apply_version_file(&mut self, vf: &VersionFile, os: &OpSys) {
        if let Some(ref ver) = vf.minecraft_version {
            self.minecraft_version = Some(ver.clone());
        }
        if let Some(ref mc) = vf.main_class {
            self.main_class = Some(mc.clone());
        }
        if let Some(ref ac) = vf.applet_class {
            self.applet_class = Some(ac.clone());
        }
        if let Some(ref args) = vf.minecraft_arguments {
            self.minecraft_arguments = Some(args.clone());
        }
        if !vf.additional_jvm_arguments.is_empty() {
            self.additional_jvm_arguments = vf.additional_jvm_arguments.clone();
        }
        if !vf.compatible_java_majors.is_empty() {
            self.compatible_java_majors = vf.compatible_java_majors.clone();
        }
        if let Some(ref assets) = vf.assets {
            self.assets = assets.clone();
        }
        if let Some(ref type_) = vf.type_ {
            self.type_ = Some(type_.clone());
        }
        if !vf.traits.is_empty() {
            for t in &vf.traits {
                if !self.traits.contains(t) {
                    self.traits.push(t.clone());
                }
            }
        }
        if !vf.add_tweakers.is_empty() {
            for t in &vf.add_tweakers {
                if !self.add_tweakers.contains(t) {
                    self.add_tweakers.push(t.clone());
                }
            }
        }
        self.required.extend(vf.required.clone());
        self.conflicts.extend(vf.conflicts.clone());

        // Libraries
        for lib in &vf.libraries {
            if lib.is_native() {
                if lib.is_active(os) {
                    self.native_libraries.push(lib.clone());
                }
            } else if lib.is_active(os) {
                self.libraries.push(lib.clone());
            }
        }

        // Main jar
        if let Some(ref mj) = vf.main_jar {
            self.main_jar = Some(mj.clone());
        }

        // Agents
        for agent in &vf.agents {
            self.agents.push(agent.clone());
        }
    }

    pub fn get_libraries_for_os(&self, os: &OpSys) -> Vec<Library> {
        let mut result: Vec<Library> = self.libraries.iter()
            .filter(|lib| lib.is_active(os) && !lib.is_native())
            .cloned()
            .collect();
        result.extend(
            self.native_libraries.iter()
                .filter(|lib| lib.is_active(os))
                .cloned()
        );
        result
    }

    pub fn get_classpath(&self, os: &OpSys) -> Vec<String> {
        let mut cp = Vec::new();
        for lib in &self.libraries {
            if lib.is_active(os) && !lib.is_native() {
                cp.push(lib.filename_for(os));
            }
        }
        if let Some(ref mj) = self.main_jar {
            cp.push(mj.filename_for(os));
        }
        cp
    }

    pub fn get_native_jars(&self, os: &OpSys) -> Vec<String> {
        self.native_libraries.iter()
            .filter(|lib| lib.is_active(os))
            .map(|lib| lib.filename_for(os))
            .collect()
    }
}

impl Default for LaunchProfile {
    fn default() -> Self {
        Self::new()
    }
}
