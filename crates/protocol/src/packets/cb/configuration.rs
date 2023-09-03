use crate::packets::prelude::*;

packets_struct! {
    PluginMessage {
        channel: String;
        data: RemainingByteArray;
    }

    Disconnect {
        reason: String;
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
        url: String;
        hash: String;
        forced: Primitive<bool>;
        prompt_message: BoolOption<String>;
    }

    FeatureFlags {
        flags: LengthVec<String>;
    }

    UpdateTags {
        todo: RemainingByteArray; // TODO
    }
}
