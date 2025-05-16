use cosmwasm_schema::cw_serde;
use once_cell::sync::Lazy;
use regex::Regex;
use std::fmt;
use std::str::FromStr;
use thiserror::Error;

/// Error type for AssetId operations
#[derive(Error, Debug, PartialEq)]
pub enum AssetIdError {
    #[error("Invalid AssetId format: {0}")]
    InvalidFormat(String),
}

/// Represents a CAIP-19 asset ID
///
/// CAIP-19 is a specification for asset type identifiers that follows the format:
/// `<chainId>/<assetNamespace>:<assetReference>`
///
/// For example: `cosmos:bbn-1/cw20:bbn17y5zvse30629t7r37xsdj73xsqp7qsdr7gpnh966wf5aslpn66rq5ekwsz`
/// represents the cBABY cw20 token on the babylon mainnet chain.
///
/// note that AssetId does not support the optional specification of Asset ID/Token ID.
#[cw_serde]
#[non_exhaustive]
pub struct AssetId {
    /// The chain ID component, CAIP-2 (e.g., "cosmos:bbn-1")
    pub chain_id: String,
    /// The asset namespace (e.g., "cw20")
    pub asset_namespace: String,
    /// The asset reference (e.g., "bbn17y5zvse30629t7r37xsdj73xsqp7qsdr7gpnh966wf5aslpn66rq5ekwsz")
    pub asset_reference: String,
}

impl AssetId {
    /// Regular expression pattern for CAIP-19 asset ID validation that matches the format roughly.
    /// It captures the chain ID, asset namespace, and asset reference.
    /// It does not validate the individual components, but ensures the overall structure is correct.
    pub fn pattern() -> &'static Regex {
        static PATTERN: Lazy<Regex> = Lazy::new(|| {
            Regex::new(r"^([-a-z0-9]+):([-_a-zA-Z0-9]+)/([-a-z0-9]+):([-.%a-zA-Z0-9]+)$").unwrap()
        });
        &PATTERN
    }

    /// Creates a new AssetId
    pub fn new(
        chain_id: &str,
        asset_namespace: &str,
        asset_reference: &str,
    ) -> Result<Self, AssetIdError> {
        // validate chain_id
        Self::validate_chain_id(chain_id)?;

        // validate asset_namespace
        Self::validate_asset_namespace(asset_namespace)?;

        // validate asset_reference
        Self::validate_asset_reference(asset_reference)?;

        // validate the whole asset_id string
        Self::validate_asset_id_str(&format!(
            "{}/{}:{}",
            chain_id, asset_namespace, asset_reference
        ))?;

        Ok(Self {
            chain_id: chain_id.to_string(),
            asset_namespace: asset_namespace.to_string(),
            asset_reference: asset_reference.to_string(),
        })
    }

    /// Decode the AssetId to CAIP-19 string
    pub fn decode(&self) -> String {
        format!(
            "{}/{}:{}",
            self.chain_id, self.asset_namespace, self.asset_reference
        )
    }

    /// Encode a CAIP-19 string into an AssetId
    pub fn encode(caip19_str: &str) -> Result<Self, AssetIdError> {
        Self::from_str(caip19_str)
    }

    /// validate the asset_id string format based on the CAIP-19 pattern
    pub fn validate_asset_id_str(s: &str) -> Result<(), AssetIdError> {
        // Check if the string matches the CAIP-19 pattern
        if !Self::pattern().is_match(s) {
            return Err(AssetIdError::InvalidFormat(s.to_string()));
        }
        Ok(())
    }

    /// validate the asset_namespace based on the CAIP-19 pattern
    pub fn validate_asset_namespace(namespace: &str) -> Result<(), AssetIdError> {
        // Check if the string matches the CAIP-19 pattern for asset_namespace
        if !Regex::new(r"^[-a-z0-9]{3,8}$")
            .expect("Failed to compile namespace regex")
            .is_match(namespace)
        {
            return Err(AssetIdError::InvalidFormat(namespace.to_string()));
        }

        Ok(())
    }

    /// validate the asset_reference based on the CAIP-19 pattern
    pub fn validate_asset_reference(reference: &str) -> Result<(), AssetIdError> {
        // Check if the string matches the CAIP-19 pattern for asset_reference
        if !Regex::new(r"^[-.%a-zA-Z0-9]{1,128}$")
            .expect("Failed to compile reference regex")
            .is_match(reference)
        {
            return Err(AssetIdError::InvalidFormat(reference.to_string()));
        }

        Ok(())
    }

    /// validate the chain_id based on the CAIP-2 pattern
    pub fn validate_chain_id(chain_id: &str) -> Result<(), AssetIdError> {
        // Check if the string matches the CAIP-2 pattern for chain_id
        if !Regex::new(r"^[-a-z0-9]{3,8}:[-_a-zA-Z0-9]{1,32}$")
            .expect("Failed to compile chain_id regex")
            .is_match(chain_id)
        {
            return Err(AssetIdError::InvalidFormat(chain_id.to_string()));
        }

        Ok(())
    }
}

impl FromStr for AssetId {
    type Err = AssetIdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // validate the asset_id string format
        Self::validate_asset_id_str(s)?;

        // Use the regex pattern to validate and capture components
        if let Some(captures) = Self::pattern().captures(s) {
            // Extract captured groups
            let chain_namespace = captures.get(1).unwrap().as_str();
            let chain_reference = captures.get(2).unwrap().as_str();
            let asset_namespace = captures.get(3).unwrap().as_str();
            let asset_reference = captures.get(4).unwrap().as_str();

            // Validate asset namespace and reference
            Self::validate_asset_namespace(asset_namespace)?;
            Self::validate_asset_reference(asset_reference)?;

            // Construct chain_id from namespace and reference
            let chain_id = format!("{}:{}", chain_namespace, chain_reference);

            Self::new(&chain_id, asset_namespace, asset_reference)
        } else {
            Err(AssetIdError::InvalidFormat(format!(
                "Asset ID '{}' does not match the expected format",
                s
            )))
        }
    }
}

impl fmt::Display for AssetId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}/{}:{}",
            self.chain_id, self.asset_namespace, self.asset_reference
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Asserts that a Result is an Err with `AssetIdError::InvalidFormat`.
    #[macro_export]
    macro_rules! assert_invalid_format {
        ($result:expr) => {
            assert!(matches!(
                $result.unwrap_err(),
                AssetIdError::InvalidFormat(_)
            ));
        };
        ($result:expr, $message:expr) => {
            assert!(
                matches!($result.unwrap_err(), AssetIdError::InvalidFormat(_)),
                $message
            );
        };
    }

    #[test]
    fn test_new_valid_asset_id() {
        // Test creating a valid AssetId using new()
        let chain_id = "cosmos:bbn-1";
        let asset_namespace = "cw20";
        let asset_reference = "bbn17y5zvse30629t7r37xsdj73xsqp7qsdr7gpnh966wf5aslpn66rq5ekwsz";

        let asset_id = AssetId::new(chain_id, asset_namespace, asset_reference).unwrap();

        assert_eq!(asset_id.chain_id, chain_id);
        assert_eq!(asset_id.asset_namespace, asset_namespace);
        assert_eq!(asset_id.asset_reference, asset_reference);
    }

    #[test]
    fn test_new_invalid_chain_id() {
        // Test with invalid chain ID format
        let result = AssetId::new("invalid-chain-id", "cw20", "token123");
        assert_invalid_format!(result);

        // Test with chain ID that's too short
        let result = AssetId::new("ab:ab", "cw20", "token123");
        assert_invalid_format!(result);

        // Test with chain ID containing invalid characters
        let result = AssetId::new("cosmo_s:bbn-1", "cw20", "token123");
        assert_invalid_format!(result);
    }

    #[test]
    fn test_new_invalid_asset_namespace() {
        // Test with asset namespace that's too short
        let result = AssetId::new("cosmos:bbn-1", "cw", "token123");
        assert_invalid_format!(result);

        // Test with asset namespace that's too long
        let result = AssetId::new("cosmos:bbn-1", "cw20cw20c", "token123");
        assert_invalid_format!(result);

        // Test with asset namespace containing uppercase
        let result = AssetId::new("cosmos:bbn-1", "CW20", "token123");
        assert_invalid_format!(result);

        // Test with asset namespace containing invalid characters
        let result = AssetId::new("cosmos:bbn-1", "cw_20", "token123");
        assert_invalid_format!(result);

        // Test with asset namespace containing whitespaces characters
        let result = AssetId::new("cosmos:bbn-1", "cw 20", "token123");
        assert_invalid_format!(result);
    }

    #[test]
    fn test_new_invalid_asset_reference() {
        // Test with asset reference that's too short
        let result = AssetId::new("cosmos:bbn-1", "cw20", "");
        assert_invalid_format!(result);

        // Test with asset reference that's too long
        let too_long_reference = "a".repeat(129);
        let result = AssetId::new("cosmos:bbn-1", "cw20", &too_long_reference);
        assert_invalid_format!(result);

        // Test with asset reference containing invalid characters
        let result = AssetId::new("cosmos:bbn-1", "cw20", "token_invalid");
        assert_invalid_format!(result);

        // Test with asset reference containing whitespace characters
        let result = AssetId::new("cosmos:bbn-1", "cw20", "token space");
        assert_invalid_format!(result);
    }

    #[test]
    fn test_decode() {
        // Test decoding an AssetId
        let asset_id = AssetId {
            chain_id: "cosmos:bbn-1".to_string(),
            asset_namespace: "cw20".to_string(),
            asset_reference: "token123".to_string(),
        };

        assert_eq!(asset_id.decode(), "cosmos:bbn-1/cw20:token123");

        // Test decoding with special characters
        let asset_id = AssetId {
            chain_id: "cosmos:bbn-1".to_string(),
            asset_namespace: "cw20".to_string(),
            asset_reference: "token-with.special%chars".to_string(),
        };

        assert_eq!(
            asset_id.decode(),
            "cosmos:bbn-1/cw20:token-with.special%chars"
        );
    }

    #[test]
    fn test_display_implementation() {
        // Test Display implementation
        let asset_id = AssetId {
            chain_id: "cosmos:bbn-1".to_string(),
            asset_namespace: "cw20".to_string(),
            asset_reference: "token123".to_string(),
        };

        assert_eq!(format!("{}", asset_id), "cosmos:bbn-1/cw20:token123");
        assert_eq!(asset_id.to_string(), "cosmos:bbn-1/cw20:token123");
    }

    #[test]
    fn test_from_str_implementation() {
        // Test FromStr implementation directly
        let caip19_str = "cosmos:bbn-1/cw20:token123";
        let asset_id = AssetId::from_str(caip19_str).unwrap();

        assert_eq!(asset_id.chain_id, "cosmos:bbn-1");
        assert_eq!(asset_id.asset_namespace, "cw20");
        assert_eq!(asset_id.asset_reference, "token123");

        // Test that FromStr and encode give the same result
        let encoded = AssetId::encode(caip19_str).unwrap();
        assert_eq!(asset_id, encoded);
    }

    #[test]
    fn test_validation_methods() {
        // Test validate_chain_id
        assert!(AssetId::validate_chain_id("cosmos:bbn-1").is_ok());
        // Test validate_chain_id error cases
        assert!(AssetId::validate_chain_id("invalid").is_err());

        // Test validate_asset_namespace
        assert!(AssetId::validate_asset_namespace("cw20").is_ok());
        // Test validate_asset_namespace error cases
        assert!(AssetId::validate_asset_namespace("cw").is_err());
        assert!(AssetId::validate_asset_namespace("CW20").is_err());

        // Test validate_asset_reference
        assert!(AssetId::validate_asset_reference("token123").is_ok());
        assert!(AssetId::validate_asset_reference("token-with.special%chars").is_ok());
        // Test validate_asset_reference error cases
        assert!(AssetId::validate_asset_reference("token$invalid").is_err());

        // Test validate_asset_id_str
        assert!(AssetId::validate_asset_id_str("cosmos:bbn-1/cw20:token123").is_ok());
        // Test validate_asset_id_str error cases
        assert!(AssetId::validate_asset_id_str("cosmos:bbn-1/cw20").is_err());
    }

    #[test]
    fn test_pattern() {
        // Test the pattern regex
        let pattern = AssetId::pattern();

        // Valid patterns
        assert!(pattern.is_match("cosmos:bbn-1/cw20:token123"));
        assert!(pattern.is_match("a:b/cw2:t"));

        // Invalid patterns
        assert!(!pattern.is_match("invalid"));
        assert!(!pattern.is_match("cosmos:bbn-1/cw20"));
        assert!(!pattern.is_match("cosmos:bbn-1/cw20:"));
    }

    #[test]
    fn test_roundtrip_conversion() {
        // Test round-trip conversion: string -> AssetId -> string
        let original = "cosmos:bbn-1/cw20:token123";
        let asset_id = AssetId::from_str(original).unwrap();
        let decoded = asset_id.decode();

        assert_eq!(original, decoded);

        // Test round-trip conversion: AssetId -> string -> AssetId
        let original_asset_id = AssetId {
            chain_id: "cosmos:bbn-1".to_string(),
            asset_namespace: "cw20".to_string(),
            asset_reference: "token123".to_string(),
        };
        let encoded = original_asset_id.decode();
        let decoded = AssetId::from_str(&encoded).unwrap();

        assert_eq!(original_asset_id, decoded);
    }

    #[test]
    fn test_encode_valid_caip19() {
        // Test with a valid CAIP-19 string
        let caip19_str =
            "cosmos:bbn-1/cw20:bbn17y5zvse30629t7r37xsdj73xsqp7qsdr7gpnh966wf5aslpn66rq5ekwsz";
        let asset_id = AssetId::encode(caip19_str).unwrap();

        assert_eq!(asset_id.chain_id, "cosmos:bbn-1");
        assert_eq!(asset_id.asset_namespace, "cw20");
        assert_eq!(
            asset_id.asset_reference,
            "bbn17y5zvse30629t7r37xsdj73xsqp7qsdr7gpnh966wf5aslpn66rq5ekwsz"
        );

        // Test that encoding the decoded AssetId gives back the original string
        assert_eq!(asset_id.decode(), caip19_str);
    }

    #[test]
    fn test_encode_invalid_components() {
        // Test with invalid asset namespace (too short)
        let result = AssetId::encode("cosmos:bbn-1/cw:token");
        assert_invalid_format!(result);

        // Test with invalid asset namespace (contains uppercase)
        let result = AssetId::encode("cosmos:bbn-1/CW20:token");
        assert_invalid_format!(result);

        // Test with invalid asset reference (contains invalid characters)
        let result = AssetId::encode("cosmos:bbn-1/cw20:token$invalid");
        assert_invalid_format!(result);

        // Test with an invalid format (missing separator)
        let result = AssetId::encode("cosmos:bbn-1cw20:token");
        assert_invalid_format!(result);

        // Test with an invalid format (wrong separator)
        let result = AssetId::encode("cosmos:bbn-1\\cw20:token");
        assert_invalid_format!(result);
    }

    #[test]
    fn test_encode_edge_cases() {
        // Test with minimum length components
        let min_caip19 = "abc:def/cw2:t";
        let min_asset_id = AssetId::encode(min_caip19).unwrap();
        assert_eq!(min_asset_id.chain_id, "abc:def");
        assert_eq!(min_asset_id.asset_namespace, "cw2");
        assert_eq!(min_asset_id.asset_reference, "t");

        // Test with maximum allowed characters in reference
        let long_reference = "a".repeat(128);
        let max_caip19 = format!("cosmos:bbn-1/cw20:{}", long_reference);
        let max_asset_id = AssetId::encode(&max_caip19).unwrap();
        assert_eq!(max_asset_id.asset_reference, long_reference);

        // Test with reference containing allowed special characters
        let special_caip19 = "cosmos:bbn-1/cw20:token-with.special%chars";
        let special_asset_id = AssetId::encode(special_caip19).unwrap();
        assert_eq!(special_asset_id.asset_reference, "token-with.special%chars");
    }

    #[test]
    fn test_encode_missing_components() {
        // Test with missing chain ID
        let result = AssetId::encode("/cw20:token");
        assert_invalid_format!(result);

        // Test with missing asset namespace
        let result = AssetId::encode("cosmos:bbn-1/:token");
        assert_invalid_format!(result);

        // Test with missing asset reference
        let result = AssetId::encode("cosmos:bbn-1/cw20:");
        assert_invalid_format!(result);

        // Test with completely empty string
        let result = AssetId::encode("");
        assert_invalid_format!(result);
    }

    #[test]
    fn test_decode_with_complex_references() {
        // Test with a reference containing multiple hyphens and dots
        let complex_ref_caip19 = "cosmos:bbn-1/cw20:token-with-multiple.dots.and-hyphens";
        let complex_ref_asset_id = AssetId::encode(complex_ref_caip19).unwrap();
        assert_eq!(
            complex_ref_asset_id.asset_reference,
            "token-with-multiple.dots.and-hyphens"
        );

        // Test with a reference containing percentage signs
        let percent_caip19 = "cosmos:bbn-1/cw20:token%20with%20encoded%20spaces";
        let percent_asset_id = AssetId::encode(percent_caip19).unwrap();
        assert_eq!(
            percent_asset_id.asset_reference,
            "token%20with%20encoded%20spaces"
        );

        // Test with a reference containing numbers
        let numeric_caip19 = "cosmos:bbn-1/cw20:123456789";
        let numeric_asset_id = AssetId::encode(numeric_caip19).unwrap();
        assert_eq!(numeric_asset_id.asset_reference, "123456789");
    }

    #[test]
    fn test_manually_composed_valid_caip19() {
        // test case is lifted from the CAIP-19 spec.
        // 2 test cases are not included due to not supporting the optional specification of Asset ID/Token ID

        // Ether Token
        let ether_token = "eip155:1/slip44:60";
        let asset_id = AssetId::from_str(ether_token).unwrap();
        assert_eq!(asset_id.chain_id, "eip155:1");
        assert_eq!(asset_id.asset_namespace, "slip44");
        assert_eq!(asset_id.asset_reference, "60");
        assert_eq!(asset_id.decode(), ether_token);

        // Bitcoin Token
        let bitcoin_token = "bip122:000000000019d6689c085ae165831e93/slip44:0";
        let asset_id = AssetId::from_str(bitcoin_token).unwrap();
        assert_eq!(asset_id.chain_id, "bip122:000000000019d6689c085ae165831e93");
        assert_eq!(asset_id.asset_namespace, "slip44");
        assert_eq!(asset_id.asset_reference, "0");
        assert_eq!(asset_id.decode(), bitcoin_token);

        // ATOM Token
        let atom_token = "cosmos:cosmoshub-3/slip44:118";
        let asset_id = AssetId::from_str(atom_token).unwrap();
        assert_eq!(asset_id.chain_id, "cosmos:cosmoshub-3");
        assert_eq!(asset_id.asset_namespace, "slip44");
        assert_eq!(asset_id.asset_reference, "118");
        assert_eq!(asset_id.decode(), atom_token);

        // Litecoin Token
        let litecoin_token = "bip122:12a765e31ffd4059bada1e25190f6e98/slip44:2";
        let asset_id = AssetId::from_str(litecoin_token).unwrap();
        assert_eq!(asset_id.chain_id, "bip122:12a765e31ffd4059bada1e25190f6e98");
        assert_eq!(asset_id.asset_namespace, "slip44");
        assert_eq!(asset_id.asset_reference, "2");
        assert_eq!(asset_id.decode(), litecoin_token);

        // Binance Token
        let binance_token = "cosmos:Binance-Chain-Tigris/slip44:714";
        let asset_id = AssetId::from_str(binance_token).unwrap();
        assert_eq!(asset_id.chain_id, "cosmos:Binance-Chain-Tigris");
        assert_eq!(asset_id.asset_namespace, "slip44");
        assert_eq!(asset_id.asset_reference, "714");
        assert_eq!(asset_id.decode(), binance_token);

        // IOV Token
        let iov_token = "cosmos:iov-mainnet/slip44:234";
        let asset_id = AssetId::from_str(iov_token).unwrap();
        assert_eq!(asset_id.chain_id, "cosmos:iov-mainnet");
        assert_eq!(asset_id.asset_namespace, "slip44");
        assert_eq!(asset_id.asset_reference, "234");
        assert_eq!(asset_id.decode(), iov_token);

        // Lisk Token
        let lisk_token = "lip9:9ee11e9df416b18b/slip44:134";
        let asset_id = AssetId::from_str(lisk_token).unwrap();
        assert_eq!(asset_id.chain_id, "lip9:9ee11e9df416b18b");
        assert_eq!(asset_id.asset_namespace, "slip44");
        assert_eq!(asset_id.asset_reference, "134");
        assert_eq!(asset_id.decode(), lisk_token);

        // DAI Token
        let dai_token = "eip155:1/erc20:0x6b175474e89094c44da98b954eedeac495271d0f";
        let asset_id = AssetId::from_str(dai_token).unwrap();
        assert_eq!(asset_id.chain_id, "eip155:1");
        assert_eq!(asset_id.asset_namespace, "erc20");
        assert_eq!(
            asset_id.asset_reference,
            "0x6b175474e89094c44da98b954eedeac495271d0f"
        );
        assert_eq!(asset_id.decode(), dai_token);

        // CryptoKitties Collection
        let cryptokitties_collection = "eip155:1/erc721:0x06012c8cf97BEaD5deAe237070F9587f8E7A266d";
        let asset_id = AssetId::from_str(cryptokitties_collection).unwrap();
        assert_eq!(asset_id.chain_id, "eip155:1");
        assert_eq!(asset_id.asset_namespace, "erc721");
        assert_eq!(
            asset_id.asset_reference,
            "0x06012c8cf97BEaD5deAe237070F9587f8E7A266d"
        );
        assert_eq!(asset_id.decode(), cryptokitties_collection);
    }
}
