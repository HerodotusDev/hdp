use core::{
    fmt::{Debug, Display},
    str::FromStr,
};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use thiserror::Error;

#[derive(PartialEq, Eq, Clone, Copy, Hash)]
pub enum ChainId {
    EthereumMainnet,
    EthereumSepolia,
    StarknetMainnet,
    StarknetSepolia,
}

#[derive(Error, Debug, PartialEq)]
#[error("Failed to parse ChainId: {input}")]
pub struct ParseChainIdError {
    input: String,
}

impl TryFrom<u128> for ChainId {
    type Error = ParseChainIdError;

    fn try_from(value: u128) -> Result<Self, Self::Error> {
        Self::from_numeric_id(value)
    }
}

impl From<ChainId> for u128 {
    fn from(chain_id: ChainId) -> Self {
        chain_id.to_numeric_id()
    }
}

impl<'de> Deserialize<'de> for ChainId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        ChainId::from_str(&s).map_err(serde::de::Error::custom)
    }
}

impl Serialize for ChainId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl FromStr for ChainId {
    type Err = ParseChainIdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "ETH_MAINNET" => Ok(Self::EthereumMainnet),
            "ETH_SEPOLIA" => Ok(Self::EthereumSepolia),
            "SN_MAINNET" => Ok(Self::StarknetMainnet),
            "SN_SEPOLIA" => Ok(Self::StarknetSepolia),
            _ => Err(ParseChainIdError {
                input: s.to_string(),
            }),
        }
    }
}

impl Display for ChainId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChainId::EthereumMainnet => write!(f, "ETH_MAINNET"),
            ChainId::EthereumSepolia => write!(f, "ETH_SEPOLIA"),
            ChainId::StarknetMainnet => write!(f, "SN_MAINNET"),
            ChainId::StarknetSepolia => write!(f, "SN_SEPOLIA"),
        }
    }
}

impl Debug for ChainId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChainId::EthereumMainnet => write!(f, "ETH_MAINNET"),
            ChainId::EthereumSepolia => write!(f, "ETH_SEPOLIA"),
            ChainId::StarknetMainnet => write!(f, "SN_MAINNET"),
            ChainId::StarknetSepolia => write!(f, "SN_SEPOLIA"),
        }
    }
}

impl ChainId {
    /// Converts the typed ChainId enum into its numeric representation.
    /// This numeric ID is used for encoding in Solidity and Cairo.
    ///
    /// # Returns
    /// A u128 representing the numeric chain ID:
    /// - 1 for Ethereum Mainnet
    /// - 11155111 for Ethereum Sepolia
    /// - 393402131332719809807700 for Starknet Mainnet
    /// - 393402133025997798000961 for Starknet Sepolia
    pub fn to_numeric_id(&self) -> u128 {
        match self {
            ChainId::EthereumMainnet => 1,
            ChainId::EthereumSepolia => 11155111,
            ChainId::StarknetMainnet => 393402131332719809807700,
            ChainId::StarknetSepolia => 393402133025997798000961,
        }
    }

    /// Converts a numeric chain ID into its corresponding ChainId enum.
    /// This method is the reverse of `to_numeric_id()`.
    ///
    /// # Arguments
    /// * `id` - A u128 representing the numeric chain ID
    ///
    /// # Returns
    /// A Result containing the corresponding ChainId enum if successful,
    /// or a ParseChainIdError if the numeric ID is not recognized.
    pub fn from_numeric_id(id: u128) -> Result<Self, ParseChainIdError> {
        match id {
            1 => Ok(Self::EthereumMainnet),
            11155111 => Ok(Self::EthereumSepolia),
            393402131332719809807700 => Ok(Self::StarknetMainnet),
            393402133025997798000961 => Ok(Self::StarknetSepolia),
            i => Err(ParseChainIdError {
                input: i.to_string(),
            }),
        }
    }

    pub fn to_be_bytes(&self) -> [u8; 16] {
        self.to_numeric_id().to_be_bytes()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_str() {
        assert_eq!(
            ChainId::from_str("ETH_MAINNET").unwrap(),
            ChainId::EthereumMainnet
        );
        assert_eq!(
            ChainId::from_str("ETH_SEPOLIA").unwrap(),
            ChainId::EthereumSepolia
        );
        assert_eq!(
            ChainId::from_str("SN_MAINNET").unwrap(),
            ChainId::StarknetMainnet
        );
        assert_eq!(
            ChainId::from_str("SN_SEPOLIA").unwrap(),
            ChainId::StarknetSepolia
        );
        assert!(ChainId::from_str("INVALID").is_err());
    }

    #[test]
    fn test_display() {
        assert_eq!(ChainId::EthereumMainnet.to_string(), "ETH_MAINNET");
        assert_eq!(ChainId::EthereumSepolia.to_string(), "ETH_SEPOLIA");
        assert_eq!(ChainId::StarknetMainnet.to_string(), "SN_MAINNET");
        assert_eq!(ChainId::StarknetSepolia.to_string(), "SN_SEPOLIA");
    }

    #[test]
    fn test_to_numeric_id() {
        assert_eq!(ChainId::EthereumMainnet.to_numeric_id(), 1);
        assert_eq!(ChainId::EthereumSepolia.to_numeric_id(), 11155111);
        assert_eq!(
            ChainId::StarknetMainnet.to_numeric_id(),
            393402131332719809807700
        );
        assert_eq!(
            ChainId::StarknetSepolia.to_numeric_id(),
            393402133025997798000961
        );
    }

    #[test]
    fn test_from_numeric_id() {
        assert_eq!(ChainId::from_numeric_id(1), Ok(ChainId::EthereumMainnet));
        assert_eq!(
            ChainId::from_numeric_id(11155111),
            Ok(ChainId::EthereumSepolia)
        );
        assert_eq!(
            ChainId::from_numeric_id(393402131332719809807700),
            Ok(ChainId::StarknetMainnet)
        );
        assert_eq!(
            ChainId::from_numeric_id(393402133025997798000961),
            Ok(ChainId::StarknetSepolia)
        );
        assert!(ChainId::from_numeric_id(999).is_err());
    }
}
