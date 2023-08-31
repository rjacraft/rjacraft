use super::*;

packets!(
    PluginMessageConfiguration {
        channel String;
        data LengthInferredVecU8;
    }

    FinishConfiguration {}

    KeepAlive {
        id i64;
    }

    Pong {
        id i64;
    }

    ResourcePack {
        result VarInt;
    }
);
