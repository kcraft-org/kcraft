use app_core::account::{AccountData, AccountTaskState};
use tracing::debug;
use tracing::info;

use crate::AuthError;
use crate::Result;

pub(crate) struct MsaTokenRefreshStep;

impl crate::AuthStep for MsaTokenRefreshStep {
    fn describe(&self) -> &str {
        "MSA Refresh"
    }

    fn perform(&mut self, data: &mut AccountData) -> Result<AccountTaskState> {
        info!("Starting silent MSA refresh");
        if data.msa_token.token.is_none() || data.msa_token.refresh_token.is_none() {
            return Err(AuthError::TokenExpired);
        }
        // Implement silent refresh here if needed
        debug!("Refresh token exists, would refresh MSA token");

        Ok(AccountTaskState::Working)
    }
}
