use crate::constant::{DEFAULT_DRY_CAIRO_RUN_CAIRO_FILE, DEFAULT_SOUND_CAIRO_RUN_CAIRO_FILE};
use crate::provider::evm::config::EvmProviderConfig;
use alloy::{primitives::ChainId, transports::http::reqwest::Url};

use std::{env, path::PathBuf};
use tokio::sync::OnceCell;

pub static CONFIG: OnceCell<HdpRunConfig> = OnceCell::const_new();

/// HdpRunConfig for the CLI
#[derive(Debug)]
pub struct HdpRunConfig {
    pub evm_provider: EvmProviderConfig,
    pub dry_run_program_path: PathBuf,
    pub sound_run_program_path: PathBuf,
    pub save_fetch_keys_file: Option<PathBuf>,
}

impl HdpRunConfig {
    pub async fn init(
        cli_rpc_url: Option<Url>,
        cli_chain_id: Option<ChainId>,
        cli_dry_run_cairo_file: Option<PathBuf>,
        cli_sound_run_cairo_file: Option<PathBuf>,
        cli_save_fetch_keys_file: Option<PathBuf>,
    ) -> &'static Self {
        let chain_id = cli_chain_id.unwrap_or_else(|| {
            env::var("CHAIN_ID")
                .expect("CHAIN_ID must be set")
                .parse()
                .expect("CHAIN_ID must be a number")
        });
        let rpc_url = cli_rpc_url.unwrap_or_else(|| {
            env::var("RPC_URL")
                .expect("RPC_URL must be set")
                .parse()
                .expect("RPC_URL must be a valid URL")
        });
        let rpc_chunk_size = env::var("RPC_CHUNK_SIZE")
            .unwrap_or_else(|_| "40".to_string())
            .parse()
            .expect("RPC_CHUNK_SIZE must be a number");
        let save_fetch_keys_file: Option<PathBuf> = cli_save_fetch_keys_file
            .or_else(|| env::var("SAVE_FETCH_KEYS_FILE").ok().map(PathBuf::from));
        let dry_run_cairo_path: PathBuf = cli_dry_run_cairo_file.unwrap_or_else(|| {
            env::var("DRY_RUN_CAIRO_PATH")
                .unwrap_or_else(|_| DEFAULT_DRY_CAIRO_RUN_CAIRO_FILE.to_string())
                .parse()
                .expect("DRY_RUN_CAIRO_PATH must be a path to a cairo file")
        });
        let sound_run_cairo_path: PathBuf = cli_sound_run_cairo_file.unwrap_or_else(|| {
            env::var("SOUND_RUN_CAIRO_PATH")
                .unwrap_or_else(|_| DEFAULT_SOUND_CAIRO_RUN_CAIRO_FILE.to_string())
                .parse()
                .expect("SOUND_RUN_CAIRO_PATH must be a path to a cairo file")
        });

        CONFIG
            .get_or_init(|| async {
                HdpRunConfig {
                    evm_provider: EvmProviderConfig {
                        rpc_url,
                        chain_id,
                        max_requests: rpc_chunk_size,
                    },
                    dry_run_program_path: dry_run_cairo_path,
                    sound_run_program_path: sound_run_cairo_path,
                    save_fetch_keys_file,
                }
            })
            .await
    }
}
