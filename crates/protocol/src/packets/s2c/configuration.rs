use crate::packets::prelude::*;

packets_struct! {
    PluginMessage {
        channel: LenString<{ 1 << 20 }>;
        data: RemainingBytes<{ 1 << 20 }>;
    }

    Disconnect {
        reason: JsonChat;
    }

    FinishConfiguration {}

    KeepAlive {
        id: Primitive<i64>;
    }

    Ping {
        id: Primitive<i64>;
    }

    RegistryData {
        todo: RemainingBytes<{ 1 << 20 }>; // TODO
    }

    ResourcePack {
        url: LenString<{ 1 << 15 }>;
        hash: LenString<40>;
        forced: Primitive<bool>;
        prompt_message: BoolOption<JsonChat>;
    }

    FeatureFlags {
        flags: LenVec<Identifier>;
    }

    UpdateTags {
        todo: RemainingBytes<{ 1 << 20 }>; // TODO
    }
}