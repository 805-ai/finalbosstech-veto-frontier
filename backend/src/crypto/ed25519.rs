// ED25519 signature generation and verification
use anyhow::{Context, Result};
use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};
use rand::rngs::OsRng;
use tracing::{info, warn};

use crate::config::Config;

pub struct Ed25519Keypair {
    pub signing_key: SigningKey,
    pub verifying_key: VerifyingKey,
}

impl Ed25519Keypair {
    pub fn generate() -> Self {
        let signing_key = SigningKey::generate(&mut OsRng);
        let verifying_key = signing_key.verifying_key();

        Ed25519Keypair {
            signing_key,
            verifying_key,
        }
    }

    pub fn sign(&self, message: &[u8]) -> Signature {
        self.signing_key.sign(message)
    }

    pub fn verify(&self, message: &[u8], signature: &Signature) -> bool {
        use ed25519_dalek::Verifier;
        self.verifying_key.verify(message, signature).is_ok()
    }

    pub fn public_key_bytes(&self) -> [u8; 32] {
        self.verifying_key.to_bytes()
    }
}

pub fn load_or_generate_keypair(config: &Config) -> Result<Ed25519Keypair> {
    // Try to load from config
    if let (Some(private_key), Some(public_key)) = (
        &config.signing_private_key,
        &config.signing_public_key,
    ) {
        // TODO: Parse keys from config (base64/hex encoded)
        warn!("Key loading from config not yet implemented, generating new keypair");
    }

    // Generate new keypair
    info!("Generating new ED25519 keypair");
    let keypair = Ed25519Keypair::generate();

    info!("Public key (base64): {}",
        base64::engine::general_purpose::STANDARD.encode(keypair.public_key_bytes()));

    Ok(keypair)
}

// Helper to use base64 crate
mod base64 {
    pub mod engine {
        pub mod general_purpose {
            use data_encoding::BASE64;

            pub struct StandardBase64;
            impl StandardBase64 {
                pub fn encode(&self, input: impl AsRef<[u8]>) -> String {
                    BASE64.encode(input.as_ref())
                }
                pub fn decode(&self, input: impl AsRef<[u8]>) -> Result<Vec<u8>, data_encoding::DecodeError> {
                    BASE64.decode(input.as_ref())
                }
            }

            pub const STANDARD: StandardBase64 = StandardBase64;
        }
    }
}
