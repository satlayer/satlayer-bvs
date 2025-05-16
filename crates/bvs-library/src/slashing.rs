use cosmwasm_schema::cw_serde;
use cosmwasm_std::{HexBinary, StdError, StdResult};
use cw_storage_plus::{Key, KeyDeserialize, Prefixer, PrimaryKey};
use std::fmt;
use std::ops::{Deref, DerefMut};

/// SlashingRequestId stores the id in hexbinary. It's a 32-byte hash of the slashing request
#[cw_serde]
pub struct SlashingRequestId(pub HexBinary);

impl SlashingRequestId {
    /// Returns the hex string representation of the slashing request id
    pub fn to_hex(&self) -> String {
        self.0.to_hex()
    }

    /// Create a SlashingRequestId from its hex string representation
    pub fn from_hex(hex: &str) -> StdResult<Self> {
        let bytes = HexBinary::from_hex(hex)?;
        if bytes.len() != 32 {
            return Err(StdError::generic_err("Invalid hex length"));
        }
        Ok(SlashingRequestId(bytes))
    }
}

impl fmt::Display for SlashingRequestId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.to_hex())
    }
}

impl Deref for SlashingRequestId {
    type Target = HexBinary;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for SlashingRequestId {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<HexBinary> for SlashingRequestId {
    fn from(bytes: HexBinary) -> Self {
        Self(bytes)
    }
}

impl From<[u8; 32]> for SlashingRequestId {
    fn from(bytes: [u8; 32]) -> Self {
        Self(HexBinary::from(bytes))
    }
}

impl PrimaryKey<'_> for SlashingRequestId {
    type Prefix = ();
    type SubPrefix = ();
    type Suffix = Self;
    type SuperSuffix = Self;

    fn key(&self) -> Vec<Key> {
        vec![Key::Ref(self.0.as_slice())]
    }
}

impl Prefixer<'_> for SlashingRequestId {
    fn prefix(&self) -> Vec<Key> {
        vec![Key::Ref(self.0.as_slice())]
    }
}

impl KeyDeserialize for SlashingRequestId {
    type Output = Self;

    const KEY_ELEMS: u16 = 1;

    #[inline(always)]
    fn from_vec(value: Vec<u8>) -> StdResult<Self::Output> {
        Ok(SlashingRequestId(HexBinary::from(value)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::MockStorage;
    use cosmwasm_std::Addr;
    use cw_storage_plus::Map;

    mod slashing_request_id {
        use super::*;

        #[test]
        fn test_from_hex_valid() {
            // Valid 32-byte hex string
            let hex = "0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20";
            let result = SlashingRequestId::from_hex(hex);
            assert!(result.is_ok());
            let id = result.unwrap();
            assert_eq!(id.to_hex(), hex);
        }

        #[test]
        fn test_from_hex_invalid_format() {
            // Invalid hex characters
            let result = SlashingRequestId::from_hex("not a hex string");
            assert!(result.is_err());

            // Valid hex but wrong length (too short)
            let result = SlashingRequestId::from_hex("0102030405");
            assert!(result.is_err());

            // Valid hex but wrong length (too long)
            let result = SlashingRequestId::from_hex(
                "0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f2021",
            );
            assert!(result.is_err());

            // Invalid hex with 0x prefix
            let result = SlashingRequestId::from_hex("0x0102030405060708090a0b0c0d0e0f");
            assert!(result.is_err());
        }

        #[test]
        fn test_to_hex() {
            let hexbinary = HexBinary::from_hex(
                "0000000000000000000000000000000000000000000000000000000000000000",
            )
            .unwrap();
            let id = SlashingRequestId::from(hexbinary.clone());
            assert_eq!(id.to_hex(), hexbinary.to_hex());
            assert_eq!(
                id.to_hex(),
                "0000000000000000000000000000000000000000000000000000000000000000"
            );

            let bytes = [
                1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
                24, 25, 26, 27, 28, 29, 30, 31, 32,
            ];
            let hexbinary = HexBinary::from(bytes);
            let id = SlashingRequestId::from(bytes);
            assert_eq!(id.to_hex(), hexbinary.to_hex());
            assert_eq!(
                id.to_hex(),
                "0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20"
            );
        }

        #[test]
        fn test_display_implementation() {
            let hex_string = "0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20";
            let id = SlashingRequestId::from_hex(hex_string).unwrap();
            assert_eq!(format!("{}", id), hex_string);
            assert_eq!(
                id.to_string(),
                "0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20"
            );
        }

        #[test]
        fn test_from_implementations() {
            // Test From<[u8; 32]>
            let bytes = [
                1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
                24, 25, 26, 27, 28, 29, 30, 31, 32,
            ];
            let id = SlashingRequestId::from(bytes);
            assert_eq!(id.0.as_slice(), &bytes);

            // Test From<HexBinary>
            let hex_binary = HexBinary::from(bytes);
            let id = SlashingRequestId::from(hex_binary.clone());
            assert_eq!(id.0, hex_binary);
        }

        #[test]
        fn test_storage_implementations() {
            let hex_string1 = "5808db357efe7b2a8c61de8c772b7e09e676a0bdb880528acfdc259c8cd1840f";
            let hex_string2 = "3dc9b61793974139e89c1fd40ba1c8da459a8fa387d8d6afb32e6386e6b42146";
            let id1 = SlashingRequestId::from_hex(hex_string1).unwrap();
            let id2 = SlashingRequestId::from_hex(hex_string2).unwrap();

            // Test PrimaryKey implementation with borrowed keys
            let map: Map<&SlashingRequestId, u64> = Map::new("test_map");
            let mut storage = MockStorage::new();

            // Store values
            map.save(&mut storage, &id1, &42u64).unwrap();
            map.save(&mut storage, &id2, &84u64).unwrap();

            // Retrieve values
            let value1 = map.load(&storage, &id1).unwrap();
            let value2 = map.load(&storage, &id2).unwrap();

            assert_eq!(value1, 42u64);
            assert_eq!(value2, 84u64);

            // Test PrimaryKey implementation
            let map: Map<SlashingRequestId, u64> = Map::new("test_map");
            let mut storage = MockStorage::new();

            // Store values
            map.save(&mut storage, id1.clone(), &42u64).unwrap();
            map.save(&mut storage, id2.clone(), &84u64).unwrap();

            // Retrieve values
            let value1 = map.load(&storage, id1).unwrap();
            let value2 = map.load(&storage, id2).unwrap();

            assert_eq!(value1, 42u64);
            assert_eq!(value2, 84u64);
        }

        #[test]
        fn test_storage_implementations_with_prefix() {
            // Create two SlashingRequestIds
            let hex_string1 = "5808db357efe7b2a8c61de8c772b7e09e676a0bdb880528acfdc259c8cd1840f";
            let hex_string2 = "3dc9b61793974139e89c1fd40ba1c8da459a8fa387d8d6afb32e6386e6b42146";
            let id1 = SlashingRequestId::from_hex(hex_string1).unwrap();
            let id2 = SlashingRequestId::from_hex(hex_string2).unwrap();

            // Create two addresses
            let addr1 = Addr::unchecked("addr1");
            let addr2 = Addr::unchecked("addr2");

            // Create a map with composite key (SlashingRequestId, &Addr)
            let map: Map<(SlashingRequestId, &Addr), u64> = Map::new("composite_key_map");
            let mut storage = MockStorage::new();

            // Store values with different combinations of keys
            map.save(&mut storage, (id1.clone(), &addr1), &100u64)
                .unwrap();
            map.save(&mut storage, (id1.clone(), &addr2), &200u64)
                .unwrap();
            map.save(&mut storage, (id2.clone(), &addr1), &300u64)
                .unwrap();
            map.save(&mut storage, (id2.clone(), &addr2), &400u64)
                .unwrap();

            // Retrieve values
            let value1 = map.load(&storage, (id1.clone(), &addr1)).unwrap();
            let value2 = map.load(&storage, (id1.clone(), &addr2)).unwrap();
            let value3 = map.load(&storage, (id2.clone(), &addr1)).unwrap();
            let value4 = map.load(&storage, (id2.clone(), &addr2)).unwrap();

            // Verify values
            assert_eq!(value1, 100u64);
            assert_eq!(value2, 200u64);
            assert_eq!(value3, 300u64);
            assert_eq!(value4, 400u64);

            // Test prefix queries - get all entries for id1
            let id1_entries: Vec<_> = map
                .prefix(id1)
                .range(&storage, None, None, cosmwasm_std::Order::Ascending)
                .collect::<StdResult<Vec<_>>>()
                .unwrap();

            // Should have 2 entries for id1
            assert_eq!(id1_entries.len(), 2);
            assert_eq!(id1_entries[0], (addr1.clone(), 100u64));
            assert_eq!(id1_entries[1], (addr2.clone(), 200u64));

            // Test prefix queries - get all entries for id2
            let id2_entries: Vec<_> = map
                .prefix(id2)
                .range(&storage, None, None, cosmwasm_std::Order::Ascending)
                .collect::<StdResult<Vec<_>>>()
                .unwrap();

            // Should have 2 entries for id2
            assert_eq!(id2_entries.len(), 2);
            assert_eq!(id2_entries[0], (addr1, 300u64));
            assert_eq!(id2_entries[1], (addr2, 400u64));
        }
    }
}
