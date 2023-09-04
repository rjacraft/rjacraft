use crate::packets::prelude::*;

packets_struct! {
    LoginStart {
        username: LenString<16>;
        uuid: ::uuid::Uuid;
    }

    EncryptionResponse {
        shared_secret: LenVec<u8>;
        verify_token: LenVec<u8>;
    }

    LoginPluginResponse {
        message_id: VarInt;
        successful: Primitive<bool>;
        data: RemainingBytes<{ 1 << 20 }>;
    }

    LoginAck {}
}
