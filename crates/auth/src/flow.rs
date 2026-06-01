use crate::{AuthError, AuthFlow, Result};
use kcraft_core::account::{AccountData, AccountTaskState, Validity};

pub enum FlowKind {
    Offline(crate::OfflineFlow),
    Msa(crate::MsaFlow),
    Yggdrasil(crate::YggdrasilFlow),
    AuthlibInjector(crate::AuthlibInjectorFlow),
}

impl FlowKind {
    pub fn execute(&mut self, data: &mut AccountData) -> Result<AccountTaskState> {
        match self {
            FlowKind::Offline(flow) => {
                data.account_type = kcraft_core::account::AccountType::Offline;
                flow.execute(data)
            }
            FlowKind::Msa(flow) => {
                data.account_type = kcraft_core::account::AccountType::Msa;
                flow.execute(data)
            }
            FlowKind::Yggdrasil(flow) => flow.execute(data),
            FlowKind::AuthlibInjector(flow) => flow.execute(data),
        }
    }

    pub fn name(&self) -> &str {
        match self {
            FlowKind::Offline(flow) => flow.name(),
            FlowKind::Msa(flow) => flow.name(),
            FlowKind::Yggdrasil(flow) => flow.name(),
            FlowKind::AuthlibInjector(flow) => flow.name(),
        }
    }
}

pub fn dispatch_refresh(data: &mut AccountData) -> Result<AccountTaskState> {
    match data.account_type {
        kcraft_core::account::AccountType::Offline => {
            let mut flow = crate::OfflineFlow::new(data.profile_name().to_string());
            flow.execute(data)
        }
        kcraft_core::account::AccountType::Msa => {
            let mut flow = crate::MsaFlow::new_silent();
            flow.execute(data)
        }
        kcraft_core::account::AccountType::AuthlibInjector => {
            if data.authlib_injector_base_url.is_empty() {
                return Err(AuthError::Auth(
                    "Authlib-Injector base URL not set".to_string(),
                ));
            }
            let mut flow = crate::AuthlibInjectorFlow::new_refresh(
                data.profile_name().to_string(),
                data.authlib_injector_base_url.clone(),
            );
            flow.execute(data)
        }
    }
}

pub fn should_refresh(data: &AccountData) -> bool {
    match data.validity {
        Validity::None => false,
        Validity::Assumed => true,
        Validity::Certain => {
            if let Some(not_after) = data.yggdrasil_token.not_after {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .ok()
                    .map(|d| d.as_secs() as i64)
                    .unwrap_or(0);
                not_after - now < 12 * 3600
            } else {
                false
            }
        }
    }
}
