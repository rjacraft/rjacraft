use crate::packets::prelude::*;

packets_struct! {
    DisconnectLogin {
        reason: JsonString<{ 1 << 18 }, Chat>;
    }

    EncryptionRequest {
        server_id: LenString<20>;
        public_key: LenVec<u8>;
        verify_token: LenVec<u8>;
    }

    LoginSuccess {
        uuid: ::uuid::Uuid;
        username: LenString<16>;
        properties: LenVec<LoginSuccessProperty>;
    }

    LoginSuccessProperty {
        name: LenString<{ 1 << 15 }>;
        value: LenString<{ 1 << 15 }>;
        signature: BoolOption<LenString<{ 1 << 15 }>>;
    }

    SetCompression {
        threshold: VarInt;
    }

    LoginPluginRequest {
        message_id: VarInt;
        channel: Identifier;
        data: RemainingBytes<{ 1 << 20 }>;
    }
}
