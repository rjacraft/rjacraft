use core::fmt;

use num_bigint::BigInt;
use sha1::Digest;

pub struct ServerHash(pub [u8; 20]);

impl ServerHash {
    pub fn new(shared_secret: [u8; 16], public_key: &impl rsa::PublicKeyParts) -> Self {
        let mut hasher = sha1::Sha1::new();

        hasher.update(b"");
        hasher.update(shared_secret);
        hasher.update(&rsa_der::public_key_to_der(
            &public_key.n().to_bytes_be(),
            &public_key.e().to_bytes_be(),
        ));

        Self(hasher.finalize().into())
    }
}

impl fmt::Display for ServerHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&BigInt::from_signed_bytes_be(&self.0).to_str_radix(16))
    }
}
