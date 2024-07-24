use std::path::PathBuf;

use hdp_primitives::constant::DEFAULT_DRY_CAIRO_RUN_CAIRO_FILE;
use hdp_provider::evm::config::EvmProviderConfig;

pub struct CompilerConfig {
    // dry-run program path
    pub dry_run_program_path: PathBuf,
    pub save_fetch_keys_file: Option<PathBuf>,
    pub provider_config: EvmProviderConfig,
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
impl Default for CompilerConfig {
    fn default() -> Self {
        let default_provider_config = EvmProviderConfig::default();
        CompilerConfig {
            dry_run_program_path: PathBuf::from(DEFAULT_DRY_CAIRO_RUN_CAIRO_FILE),
            provider_config: default_provider_config,
            save_fetch_keys_file: None,
        }
    }
}
