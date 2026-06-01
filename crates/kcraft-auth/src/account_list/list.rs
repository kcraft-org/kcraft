use std::path::PathBuf;

use kcraft_core::account::Validity;
use tracing::debug;

use crate::{AuthError, Result};

use super::account::MinecraftAccount;
use super::serialize::AccountFile;

#[derive(Debug, Clone)]
pub struct AccountList {
    accounts: Vec<MinecraftAccount>,
    default_account: Option<usize>,
    file_path: PathBuf,
    autosave: bool,
}

impl AccountList {
    pub fn new(file_path: PathBuf) -> Self {
        let mut list = AccountList {
            accounts: Vec::new(),
            default_account: None,
            file_path,
            autosave: false,
        };
        let _ = list.load();
        list
    }

    pub fn count(&self) -> usize {
        self.accounts.len()
    }

    pub fn at(&self, index: usize) -> Option<&MinecraftAccount> {
        self.accounts.get(index)
    }

    pub fn get_account_by_profile_name(&self, name: &str) -> Option<&MinecraftAccount> {
        self.accounts.iter().find(|a| a.data.profile_name() == name)
    }

    pub fn get_account_by_profile_id(&self, id: &str) -> Option<&MinecraftAccount> {
        self.accounts.iter().find(|a| a.data.profile_id() == id)
    }

    pub fn add_account(&mut self, account: MinecraftAccount) {
        self.accounts.push(account);
        if self.autosave {
            let _ = self.save();
        }
    }

    pub fn remove_account(&mut self, index: usize) {
        if index < self.accounts.len() {
            self.accounts.remove(index);
            if self.default_account == Some(index) {
                self.default_account = None;
            } else if let Some(default) = self.default_account {
                if index < default {
                    self.default_account = Some(default - 1);
                }
            }
            if self.autosave {
                let _ = self.save();
            }
        }
    }

    pub fn default_account(&self) -> Option<&MinecraftAccount> {
        self.default_account.and_then(|i| self.accounts.get(i))
    }

    pub fn set_default_account(&mut self, index: usize) {
        if index < self.accounts.len() {
            if let Some(old) = self.default_account {
                if old < self.accounts.len() {
                    self.accounts[old].active = false;
                }
            }
            self.accounts[index].active = true;
            self.default_account = Some(index);
            if self.autosave {
                let _ = self.save();
            }
        }
    }

    pub fn profile_names(&self) -> Vec<String> {
        self.accounts
            .iter()
            .map(|a| a.data.profile_name().to_string())
            .collect()
    }

    pub fn any_account_is_valid(&self) -> bool {
        self.accounts
            .iter()
            .any(|a| a.data.validity == Validity::Certain)
    }

    pub fn load(&mut self) -> Result<()> {
        if !self.file_path.exists() {
            return Ok(());
        }
        let data = std::fs::read_to_string(&self.file_path)
            .map_err(|e| AuthError::Network(format!("Failed to read accounts file: {}", e)))?;

        let file: AccountFile =
            serde_json::from_str(&data).map_err(|e| AuthError::Serialization(e.to_string()))?;

        self.accounts.clear();
        self.default_account = None;

        for (i, json) in file.accounts.iter().enumerate() {
            if let Some(account) = MinecraftAccount::load_from_json(json) {
                let is_active = json
                    .get("active")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                if is_active {
                    self.default_account = Some(i);
                }
                self.accounts.push(account);
            }
        }

        debug!(
            "Loaded {} accounts from {}",
            self.accounts.len(),
            self.file_path.display()
        );
        Ok(())
    }

    pub fn save(&self) -> Result<()> {
        let accounts: Vec<serde_json::Value> =
            self.accounts.iter().map(|a| a.save_to_json()).collect();

        let file = AccountFile {
            format_version: 3,
            accounts,
        };

        let data = serde_json::to_string_pretty(&file)
            .map_err(|e| AuthError::Serialization(e.to_string()))?;

        if let Some(parent) = self.file_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        kcraft_fs::write(&self.file_path, data.as_bytes())
            .map_err(|e| AuthError::Network(e.to_string()))?;

        debug!(
            "Saved {} accounts to {}",
            self.accounts.len(),
            self.file_path.display()
        );
        Ok(())
    }

    pub fn set_autosave(&mut self, autosave: bool) {
        self.autosave = autosave;
    }
}
