use crate::packets::prelude::*;

packets_struct! {
    PluginMessage {
        channel: LenString<{ 1 << 20 }>;
        data: RemainingByteArray;
    }

    Disconnect {
        reason: JsonString<{ 1 << 18 }, Chat>;
    }

    FinishConfiguration {}

    KeepAlive {
        id: Primitive<i64>;
    }

    Ping {
        id: Primitive<i64>;
    }

    RegistryData {
        todo: RemainingByteArray; // TODO
    }

    ResourcePack {
        url: LenString<{ 1 << 15 }>;
        hash: LenString<40>;
        forced: Primitive<bool>;
        prompt_message: BoolOption<JsonString<{ 1 << 18 }, Chat>>;
    }

    FeatureFlags {
        flags: LenVec<Identifier>;
    }

    UpdateTags {
        todo: RemainingByteArray; // TODO
    }
}
