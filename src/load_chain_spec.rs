use crate::chain_spec;
pub use chain_spec::IsV1Network;
use polymesh_runtime_testnet_v1::config::GenesisConfig;
/// The chain specification (this should eventually be replaced by a more general JSON-based chain
/// specification).
#[derive(Clone, Debug)]
pub enum ChainType {
    /// Whatever the current runtime is, with just Alice as an auth.
    Development,
    /// Whatever the current runtime is, with simple Alice/Bob auths.
    Local,
    /// Whatever the current runtime is, with some more auths.
    Live,
    /// V1 runtime with development configuration
    V1Development,
    /// V1 runtime with local configuration.
    V1Local,
    /// V1 runtime with live configuration.
    V1Live,
}

impl Default for ChainType {
    fn default() -> Self {
        ChainType::Development
    }
}

/// Get a chain config from a spec setting.
impl ChainType {
    pub(crate) fn load(self) -> Result<sc_service::ChainSpec<GenesisConfig>, String> {
        match self {
            ChainType::Development => Ok(chain_spec::general_development_testnet_config()),
            ChainType::Local => Ok(chain_spec::general_local_testnet_config()),
            ChainType::Live => Ok(chain_spec::general_live_testnet_config()),
            ChainType::V1Development => Ok(chain_spec::v1_develop_testnet_config()),
            ChainType::V1Local => Ok(chain_spec::v1_local_testnet_config()),
            ChainType::V1Live => Ok(chain_spec::v1_live_testnet_config()),
        }
    }

    pub(crate) fn from(s: &str) -> Option<Self> {
        match s {
            "dev" => Some(ChainType::Development),
            "local" => Some(ChainType::Local),
            "live" => Some(ChainType::Live),
            "v1-dev" => Some(ChainType::V1Development),
            "v1-local" => Some(ChainType::V1Local),
            "v1-live" => Some(ChainType::V1Live),
            "" => Some(ChainType::default()),
            _ => None,
        }
    }
}

/// Load the `ChainType` for the given `id`.
pub fn load_spec(id: &str) -> Result<Option<sc_service::ChainSpec<GenesisConfig>>, String> {
    Ok(match ChainType::from(id) {
        Some(spec) => Some(spec.load()?),
        None => None,
    })
}
