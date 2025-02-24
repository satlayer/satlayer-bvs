use base64::{engine::general_purpose, Engine as _};
use bech32::{self, ToBase32, Variant};
use cosmwasm_std::Addr;
use ripemd::Ripemd160;
use secp256k1::ecdsa::Signature;
use secp256k1::{Message, PublicKey, Secp256k1, SecretKey};
use sha2::{Digest, Sha256};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Account {
    secret_key: SecretKey,
    pub public_key: PublicKey,
    pub address: Addr,
}

impl Account {
    // create new account
    pub fn new(s: String) -> Self {
        // Create a new Secp256k1 context
        let secp = Secp256k1::new();

        // convert string to vec
        let seed = Sha256::digest(s.as_bytes()).to_vec();

        // Generate a random secret key
        let secret_key = SecretKey::from_slice(&seed).unwrap();

        // Derive the public key from the secret key
        let public_key = PublicKey::from_secret_key(&secp, &secret_key);

        // Serialize the public key
        let public_key_bytes = public_key.serialize();

        // Compute the SHA-256 hash of the public key
        let sha256_result = Sha256::digest(public_key_bytes);

        // Compute the RIPEMD-160 hash of the SHA-256 hash
        let ripemd160_result = Ripemd160::digest(sha256_result);

        // Encode the RIPEMD-160 hash as a Bech32 address
        let address = bech32::encode("bbn", ripemd160_result.to_base32(), Variant::Bech32).unwrap();

        Account {
            secret_key,
            public_key,
            address: Addr::unchecked(address),
        }
    }

    /// Sign the message hash with ECDSA
    ///
    /// use message_hash to convert string to 32-byte hash or use custom implementation
    pub fn sign(&self, message_hash: Vec<u8>) -> Signature {
        // Create a new Secp256k1 signing only context
        let secp = Secp256k1::signing_only();

        // Convert the message_hash to a 32-byte array
        let hash_bytes: [u8; 32] = message_hash
            .as_slice()
            .try_into()
            .expect("hash length is not 32 bytes");

        // Sign the hash with the secret key
        secp.sign_ecdsa(&Message::from_digest(hash_bytes), &self.secret_key)
    }

    /// Create 32-byte hash of the message
    ///
    /// This is a default implementation to create 32-byte hash of the message
    pub fn message_hash(message: String) -> Vec<u8> {
        // Create SHA-256 hash of the message and convert it to a 32-byte array
        let hash = Sha256::digest(message.as_bytes());
        hash.to_vec()
    }

    // Return base64 encoding format of public key
    pub fn public_key_base64(&self) -> String {
        general_purpose::STANDARD.encode(&self.public_key.serialize())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_crypto::secp256k1_verify;

    #[test]
    fn test_new() {
        let account = Account::new("seed".to_string());
        assert_eq!(
            account.public_key.to_string(),
            "02c8f031561c4758c9551cff47246f2c347189fe684c04da35cf88e813f810e3c2"
        );
        assert_eq!(
            account.address.to_string(),
            "bbn1efqyslkz34qurfjajpruzwv5v22c65kq3uugqf"
        );
    }

    #[test]
    fn test_sign() {
        let account = Account::new("seed".to_string());
        let message = "hello".to_string();
        let message_hash = Account::message_hash(message.clone());
        let signature = account.sign(message_hash.clone());
        assert_eq!(signature.to_string(), "3044022067e20eddd4e86a76c80382e80852ffbccd3131962070ee1ad524a798be5d83cb022000d7a45e0faa08dee805381df8c4a7ead53f8e319c69bd67fa3cc031ef519cbb");

        // verify the signature + public key with secp256k1_verify
        let verify_res = secp256k1_verify(
            &message_hash,
            &signature.serialize_compact(),
            &account.public_key.serialize(),
        )
        .unwrap();
        assert_eq!(verify_res, true);
    }
}
