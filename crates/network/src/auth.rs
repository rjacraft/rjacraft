use std::result;

use bevy_ecs::prelude::*;
use rand::RngCore;
use rjacraft_protocol::{error, packets::*, types::*, ProtocolType};
use rsa::PublicKeyParts;
use tokio::sync::oneshot;

use crate::network::B2nEvent;

pub type Result = result::Result<(Uuid, player_info::Profile), Text>;

#[derive(Debug, thiserror::Error)]
pub enum EncryptError {
    #[error("RSA error")]
    Rsa(#[from] rsa::errors::Error),
    #[error("Nonce mismatch")]
    Nonce,
    #[error("Shared secret length")]
    SharedSecret(error::Overrun<16>),
    #[error("Client disconnected")]
    Channel(#[from] oneshot::error::RecvError),
}

pub struct Handle {
    pub peer: Entity,
    pub uuid: Uuid,
    pub username: rjacraft_authlib::profile::Name,
    pub(crate) b2n: flume::Sender<B2nEvent>,
}

impl Handle {
    pub fn compress(&self, threshold: Option<u32>) {
        let _ = self.b2n.send(B2nEvent::Compress(threshold));
    }

    pub async fn encrypt(
        &mut self,
        key: &rsa::RSAPrivateKey,
    ) -> result::Result<[u8; 16], EncryptError> {
        let mut nonce = [0; 4];
        rand::thread_rng().fill_bytes(&mut nonce);

        let (tx, rx) = oneshot::channel();

        let _ = self.b2n.send(B2nEvent::NeedEncryptionResponse(tx));
        let _ = self.b2n.send(B2nEvent::Packet(
            s2c::LoginPacket::ServerIdentity {
                server_id: String::new().try_into().unwrap(),
                public_key: rsa_der::public_key_to_der(
                    &key.n().to_bytes_be(),
                    &key.e().to_bytes_be(),
                )
                .into(),
                nonce: Vec::from(&nonce[..]).into(),
            }
            .to_bytes_expect(),
        ));

        let (shared_secret_enc, nonce_enc) = rx.await?;

        if key.decrypt(rsa::PaddingScheme::PKCS1v15Encrypt, &nonce_enc)? != nonce {
            Err(EncryptError::Nonce)
        } else {
            let shared_secret =
                key.decrypt(rsa::PaddingScheme::PKCS1v15Encrypt, &shared_secret_enc)?;
            let shared_secret: [u8; 16] = shared_secret
                .as_slice()
                .try_into()
                .map_err(|_| EncryptError::SharedSecret(error::Overrun(shared_secret.len())))?;

            let _ = self.b2n.send(B2nEvent::Encrypt(shared_secret));

            Ok(shared_secret)
        }
    }
}
