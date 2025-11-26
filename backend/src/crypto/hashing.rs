// SHA3-512 hashing for receipts
use sha3::{Digest, Sha3_512};

pub fn sha3_512_hash(data: &[u8]) -> String {
    let mut hasher = Sha3_512::new();
    hasher.update(data);
    let result = hasher.finalize();
    hex::encode(result)
}

pub fn sha3_512_hash_str(data: &str) -> String {
    sha3_512_hash(data.as_bytes())
}

// Helper hex encoding
mod hex {
    pub fn encode(bytes: impl AsRef<[u8]>) -> String {
        data_encoding::HEXLOWER.encode(bytes.as_ref())
    }
}
