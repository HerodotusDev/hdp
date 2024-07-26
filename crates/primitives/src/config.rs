use std::{collections::HashMap, fs, path::PathBuf};

use serde::Deserialize;

pub const DEFAULT_DRY_CAIRO_RUN_CAIRO_FILE: &str = "build/contract_dry_run.json";
pub const DEFAULT_SOUND_CAIRO_RUN_CAIRO_FILE: &str = "build/hdp.json";
pub const HDP_CONFIG_FILE: &str = "hdp_config.json";

pub type AllChainConfigs = HashMap<u64, ChainConfig>;

/// Configuration for the CLI
#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(default = "default_dry_run_program_path")]
    pub dry_run_program_path: PathBuf,
    #[serde(default = "default_sound_run_program_path")]
    pub sound_run_program_path: PathBuf,
    #[serde(default = "default_save_fetch_keys_file")]
    pub save_fetch_keys_file: Option<PathBuf>,
    pub chains: HashMap<u64, ChainConfig>,
}

impl Config {
    pub async fn init(
        cli_dry_run_cairo_file: Option<PathBuf>,
        cli_sound_run_cairo_file: Option<PathBuf>,
        cli_save_fetch_keys_file: Option<PathBuf>,
    ) -> Self {
        let config_content =
            fs::read_to_string(HDP_CONFIG_FILE).expect("Failed to read config file");
        let mut config: Config =
            serde_json::from_str(&config_content).expect("Failed to parse config file");

        // Override config with arguments if provided
        if cli_dry_run_cairo_file.is_some() {
            config.dry_run_program_path = cli_dry_run_cairo_file.unwrap();
        }
        if cli_sound_run_cairo_file.is_some() {
            config.sound_run_program_path = cli_sound_run_cairo_file.unwrap();
        }
        if cli_save_fetch_keys_file.is_some() {
            config.save_fetch_keys_file = cli_save_fetch_keys_file;
        }

        config
    }
}

fn default_save_fetch_keys_file() -> Option<PathBuf> {
    None
}

fn default_dry_run_program_path() -> PathBuf {
    DEFAULT_DRY_CAIRO_RUN_CAIRO_FILE.into()
}

fn default_sound_run_program_path() -> PathBuf {
    DEFAULT_SOUND_CAIRO_RUN_CAIRO_FILE.into()
}

#[derive(Debug, Deserialize)]
pub struct ChainConfig {
    pub rpc_url: String,
    #[serde(default = "default_rpc_chunk_size")]
    pub rpc_chunk_size: u32,
}

fn default_rpc_chunk_size() -> u32 {
    100
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn test_deserde() {
        let config = r#"
    {
        "dry_run_program_path": "build/contract_dry_run.json",
        "sound_run_program_path": "build/hdp.json",
        "chains": {
            "1": {
                "rpc_url": "http://localhost:8545",
                "rpc_chunk_size": 2000
            },
            "11155111": {
                "rpc_url": "http://localhost:3000"
            }
        }
    }
    "#;

        let config: Config = serde_json::from_str(config).unwrap();
        assert_eq!(config.chains.len(), 2);
        assert_eq!(
            config.chains.get(&1).unwrap().rpc_url,
            "http://localhost:8545"
        );
        assert_eq!(config.chains.get(&1).unwrap().rpc_chunk_size, 2000);
        assert_eq!(
            config.chains.get(&11155111).unwrap().rpc_url,
            "http://localhost:3000"
        );
        assert_eq!(config.chains.get(&11155111).unwrap().rpc_chunk_size, 100);
        assert_eq!(config.save_fetch_keys_file, None);
        assert_eq!(
            config.dry_run_program_path,
            PathBuf::from_str("build/contract_dry_run.json").unwrap()
        );
        assert_eq!(
            config.sound_run_program_path,
            PathBuf::from_str("build/hdp.json").unwrap()
        );
    }
}
