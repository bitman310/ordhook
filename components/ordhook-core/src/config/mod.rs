use crate::core::OrdhookConfig;
pub use chainhook_sdk::indexer::IndexerConfig;
use chainhook_sdk::observer::EventObserverConfig;
use chainhook_sdk::types::{
    BitcoinBlockSignaling, BitcoinNetwork, StacksNetwork, StacksNodeConfig,
};
use std::path::PathBuf;

const DEFAULT_MAINNET_ORDINALS_SQLITE_ARCHIVE: &str =
    "https://archive.hiro.so/mainnet/ordhook/mainnet-ordhook-sqlite-latest";

pub const DEFAULT_INGESTION_PORT: u16 = 20455;
pub const DEFAULT_CONTROL_PORT: u16 = 20456;
pub const DEFAULT_ULIMIT: usize = 2048;
pub const DEFAULT_MEMORY_AVAILABLE: usize = 8;
pub const DEFAULT_BITCOIND_RPC_THREADS: usize = 4;
pub const DEFAULT_BITCOIND_RPC_TIMEOUT: u32 = 15;

#[derive(Clone, Debug)]
pub struct Config {
    pub storage: StorageConfig,
    pub http_api: PredicatesApi,
    pub resources: ResourcesConfig,
    pub network: IndexerConfig,
    pub snapshot: SnapshotConfig,
    pub logs: LogConfig,
}

#[derive(Clone, Debug)]
pub struct LogConfig {
    pub ordinals_internals: bool,
    pub chainhook_internals: bool,
}

#[derive(Clone, Debug)]
pub struct StorageConfig {
    pub working_dir: String,
}

#[derive(Clone, Debug)]
pub enum PredicatesApi {
    Off,
    On(PredicatesApiConfig),
}

#[derive(Clone, Debug)]
pub struct PredicatesApiConfig {
    pub http_port: u16,
    pub display_logs: bool,
}

#[derive(Clone, Debug)]
pub enum SnapshotConfig {
    Build,
    Download(String),
}

#[derive(Clone, Debug)]
pub struct PathConfig {
    pub file_path: PathBuf,
}

#[derive(Clone, Debug)]
pub struct UrlConfig {
    pub file_url: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ResourcesConfig {
    pub ulimit: usize,
    pub cpu_core_available: usize,
    pub memory_available: usize,
    pub bitcoind_rpc_threads: usize,
    pub bitcoind_rpc_timeout: u32,
    pub expected_observers_count: usize,
}

impl ResourcesConfig {
    pub fn get_optimal_thread_pool_capacity(&self) -> usize {
        // Generally speaking when dealing a pool, we need one thread for
        // feeding the thread pool and eventually another thread for
        // handling the "reduce" step.
        self.cpu_core_available.saturating_sub(2).max(1)
    }
}

impl Config {
    pub fn is_http_api_enabled(&self) -> bool {
        match self.http_api {
            PredicatesApi::Off => false,
            PredicatesApi::On(_) => true,
        }
    }

    pub fn get_ordhook_config(&self) -> OrdhookConfig {
        OrdhookConfig {
            resources: self.resources.clone(),
            db_path: self.expected_cache_path(),
            first_inscription_height: match self.network.bitcoin_network {
                BitcoinNetwork::Mainnet => 767430,
                BitcoinNetwork::Regtest => 1,
                BitcoinNetwork::Testnet => 2413343,
                BitcoinNetwork::Signet => 112402,
            },
            logs: self.logs.clone(),
        }
    }

    pub fn get_event_observer_config(&self) -> EventObserverConfig {
        EventObserverConfig {
            bitcoin_rpc_proxy_enabled: true,
            chainhook_config: None,
            ingestion_port: DEFAULT_INGESTION_PORT,
            bitcoind_rpc_username: self.network.bitcoind_rpc_username.clone(),
            bitcoind_rpc_password: self.network.bitcoind_rpc_password.clone(),
            bitcoind_rpc_url: self.network.bitcoind_rpc_url.clone(),
            bitcoin_block_signaling: self.network.bitcoin_block_signaling.clone(),
            display_logs: false,
            cache_path: self.storage.working_dir.clone(),
            bitcoin_network: self.network.bitcoin_network.clone(),
            stacks_network: self.network.stacks_network.clone(),
            data_handler_tx: None,
        }
    }

    pub fn should_bootstrap_through_download(&self) -> bool {
        match &self.snapshot {
            SnapshotConfig::Build => false,
            SnapshotConfig::Download(_) => true,
        }
    }

    pub fn expected_api_config(&self) -> &PredicatesApiConfig {
        match self.http_api {
            PredicatesApi::On(ref config) => config,
            _ => unreachable!(),
        }
    }

    pub fn expected_cache_path(&self) -> PathBuf {
        let mut destination_path = PathBuf::new();
        destination_path.push(&self.storage.working_dir);
        destination_path
    }

    fn expected_remote_ordinals_sqlite_base_url(&self) -> &str {
        match &self.snapshot {
            SnapshotConfig::Build => unreachable!(),
            SnapshotConfig::Download(url) => &url,
        }
    }

    pub fn expected_remote_ordinals_sqlite_sha256(&self) -> String {
        format!("{}.sha256", self.expected_remote_ordinals_sqlite_base_url())
    }

    pub fn expected_remote_ordinals_sqlite_url(&self) -> String {
        format!("{}.tar.gz", self.expected_remote_ordinals_sqlite_base_url())
    }

    pub fn devnet_default() -> Config {
        Config {
            storage: StorageConfig {
                working_dir: default_cache_path(),
            },
            http_api: PredicatesApi::Off,
            snapshot: SnapshotConfig::Build,
            resources: ResourcesConfig {
                cpu_core_available: num_cpus::get(),
                memory_available: DEFAULT_MEMORY_AVAILABLE,
                ulimit: DEFAULT_ULIMIT,
                bitcoind_rpc_threads: DEFAULT_BITCOIND_RPC_THREADS,
                bitcoind_rpc_timeout: DEFAULT_BITCOIND_RPC_TIMEOUT,
                expected_observers_count: 1,
            },
            network: IndexerConfig {
                bitcoind_rpc_url: "http://0.0.0.0:18443".into(),
                bitcoind_rpc_username: "devnet".into(),
                bitcoind_rpc_password: "devnet".into(),
                bitcoin_block_signaling: BitcoinBlockSignaling::Stacks(
                    StacksNodeConfig::default_localhost(DEFAULT_INGESTION_PORT),
                ),
                stacks_network: StacksNetwork::Devnet,
                bitcoin_network: BitcoinNetwork::Regtest,
            },
            logs: LogConfig {
                ordinals_internals: true,
                chainhook_internals: false,
            },
        }
    }

    pub fn testnet_default() -> Config {
        Config {
            storage: StorageConfig {
                working_dir: default_cache_path(),
            },
            http_api: PredicatesApi::Off,
            snapshot: SnapshotConfig::Build,
            resources: ResourcesConfig {
                cpu_core_available: num_cpus::get(),
                memory_available: DEFAULT_MEMORY_AVAILABLE,
                ulimit: DEFAULT_ULIMIT,
                bitcoind_rpc_threads: DEFAULT_BITCOIND_RPC_THREADS,
                bitcoind_rpc_timeout: DEFAULT_BITCOIND_RPC_TIMEOUT,
                expected_observers_count: 1,
            },
            network: IndexerConfig {
                bitcoind_rpc_url: "http://0.0.0.0:18332".into(),
                bitcoind_rpc_username: "devnet".into(),
                bitcoind_rpc_password: "devnet".into(),
                bitcoin_block_signaling: BitcoinBlockSignaling::Stacks(
                    StacksNodeConfig::default_localhost(DEFAULT_INGESTION_PORT),
                ),
                stacks_network: StacksNetwork::Testnet,
                bitcoin_network: BitcoinNetwork::Testnet,
            },
            logs: LogConfig {
                ordinals_internals: true,
                chainhook_internals: false,
            },
        }
    }

    pub fn mainnet_default() -> Config {
        Config {
            storage: StorageConfig {
                working_dir: default_cache_path(),
            },
            http_api: PredicatesApi::Off,
            snapshot: SnapshotConfig::Download(DEFAULT_MAINNET_ORDINALS_SQLITE_ARCHIVE.to_string()),
            resources: ResourcesConfig {
                cpu_core_available: num_cpus::get(),
                memory_available: DEFAULT_MEMORY_AVAILABLE,
                ulimit: DEFAULT_ULIMIT,
                bitcoind_rpc_threads: DEFAULT_BITCOIND_RPC_THREADS,
                bitcoind_rpc_timeout: DEFAULT_BITCOIND_RPC_TIMEOUT,
                expected_observers_count: 1,
            },
            network: IndexerConfig {
                bitcoind_rpc_url: "http://0.0.0.0:8332".into(),
                bitcoind_rpc_username: "devnet".into(),
                bitcoind_rpc_password: "devnet".into(),
                bitcoin_block_signaling: BitcoinBlockSignaling::Stacks(
                    StacksNodeConfig::default_localhost(DEFAULT_INGESTION_PORT),
                ),
                stacks_network: StacksNetwork::Mainnet,
                bitcoin_network: BitcoinNetwork::Mainnet,
            },
            logs: LogConfig {
                ordinals_internals: true,
                chainhook_internals: false,
            },
        }
    }
}

pub fn default_cache_path() -> String {
    let mut cache_path = std::env::current_dir().expect("unable to get current dir");
    cache_path.push("ordhook");
    format!("{}", cache_path.display())
}
