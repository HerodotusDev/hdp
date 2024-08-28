use std::{collections::HashMap, path::PathBuf};

#[cfg(feature = "test_utils")]
use crate::constant::DEFAULT_DRY_CAIRO_RUN_CAIRO_FILE;
use crate::provider::config::ProviderConfig;

pub struct CompilerConfig {
    // dry-run program path
    pub dry_run_program_path: PathBuf,
    pub save_fetch_keys_file: Option<PathBuf>,
    // chain_id => provider config
    pub provider_config: HashMap<u64, ProviderConfig>,
}

impl CompilerConfig {
    pub fn with_dry_run_program_path(self, dry_run_program_path: PathBuf) -> Self {
        Self {
            dry_run_program_path,
            provider_config: self.provider_config,
            save_fetch_keys_file: self.save_fetch_keys_file,
        }
    }
}

// Default config for the compiler only for testing
#[cfg(feature = "test_utils")]
impl Default for CompilerConfig {
    fn default() -> Self {
        CompilerConfig {
            dry_run_program_path: PathBuf::from(DEFAULT_DRY_CAIRO_RUN_CAIRO_FILE),
            provider_config: [(
                ProviderConfig::default().chain_id,
                ProviderConfig::default(),
            )]
            .into(),
            save_fetch_keys_file: None,
        }
    }
}
