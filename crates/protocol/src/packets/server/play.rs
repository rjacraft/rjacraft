use super::*;

packets!(

    JoinGame {
        entity_id i32;
        is_hardcore bool;
        dimensions VarIntPrefixedVec<String>;
        max_players VarInt;
        view_distance VarInt;
        simulation_distance VarInt;
        reduced_debug_info bool;
        enable_respawn_screen bool;
        dimension_type String;
        dimension_name String;
        hashed_seed i64;
        gamemde u8;
        previous_gamemode i8;
        is_debug bool;
        is_flat bool;
        death_info Option<JoinGameDeathInfo>;
        portal_cooldown VarInt;
    },

    JoinGameDeathInfo {
        dimension_name String;
        position u64;
    }

);
