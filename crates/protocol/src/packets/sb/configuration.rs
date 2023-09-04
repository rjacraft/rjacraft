use crate::packets::prelude::*;

packets_struct! {
    PluginMessageConfiguration {
        channel: Identifier;
        data: RemainingByteArray<{ 1 << 20 }>;
    }

    FinishConfiguration {}

    KeepAlive {
        id: Primitive<i64>;
    }

    Pong {
        id: Primitive<i64>;
    }

    ResourcePack {
        result: VarInt;
    }
}
