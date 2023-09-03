use crate::packets::prelude::*;

packets_struct! {
    LoginStart {
        username: String;
        uuid: ::uuid::Uuid;
    }

    EncryptionResponse {
        shared_secret: LengthVec<u8>;
        verify_token: LengthVec<u8>;
    }

    LoginPluginResponse {
        message_id: VarInt;
        successful: Primitive<bool>;
        data: RemainingByteArray;
    }

    LoginAck {}
}
