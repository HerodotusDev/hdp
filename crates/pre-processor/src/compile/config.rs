use std::path::PathBuf;

use hdp_primitives::config::AllChainConfigs;

pub struct CompilerConfig {
    // dry-run program path
    pub dry_run_program_path: PathBuf,
    pub save_fetch_keys_file: Option<PathBuf>,
    pub provider_config: AllChainConfigs,
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
