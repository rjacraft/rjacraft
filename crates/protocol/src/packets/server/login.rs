use super::*;

packets!(
    DisconnectLogin {
        reason String;
    }

    EncryptionRequest {
        server_id String;
        public_key VarIntPrefixedVec<u8>;
        verify_token VarIntPrefixedVec<u8>;
    }

    LoginSuccess {
        uuid Uuid;
        username String;
        properties VarIntPrefixedVec<LoginSuccessProperty>;
    }

    LoginSuccessProperty {
        name String;
        value String;
        signature Option<String>;
    }

    SetCompression {
        threshold VarInt;
    }

    LoginPluginRequest {
        message_id VarInt;
        channel String;
        data LengthInferredVecU8;
    }
);
