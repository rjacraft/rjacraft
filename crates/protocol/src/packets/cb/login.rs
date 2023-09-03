use crate::packets::prelude::*;

packets_struct! {
    DisconnectLogin {
        reason: String;
    }

    EncryptionRequest {
        server_id: String;
        public_key: LengthVec<u8>;
        verify_token: LengthVec<u8>;
    }

    LoginSuccess {
        uuid: ::uuid::Uuid;
        username: String;
        properties: LengthVec<LoginSuccessProperty>;
    }

    LoginSuccessProperty {
        name: String;
        value: String;
        signature: BoolOption<String>;
    }

    SetCompression {
        threshold: VarInt;
    }

    LoginPluginRequest {
        message_id: VarInt;
        channel: String;
        data: RemainingByteArray;
    }
}
