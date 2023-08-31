use super::*;

packets! {
    PluginMessage {
        channel String;
        data LengthInferredVecU8;
    }

    Disconnect {
        reason String;
    }

    FinishConfiguration {
    }

    KeepAlive {
        id i64;
    }

    Ping {
        id i64;
    }

    RegistryData {
        todo LengthInferredVecU8 // TODO
    }

    ResourcePack {
        url String;
        hash String;
        forced bool;
        promt_message Option<String>
    }

    FeatureFlags {
        flags VarIntPrefixedVec<String>
    }

    UpdateTags {
        todo LengthInferredVecU8 // TODO
    }
}
