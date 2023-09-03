use crate::packets::prelude::*;

packets_struct! {
    JoinGame {
        entity_id: Primitive<i32>;
        is_hardcore: Primitive<bool>;
        dimensions: LengthVec<String>;
        max_players: VarInt;
        view_distance: VarInt;
        simulation_distance: VarInt;
        reduced_debug_info: Primitive<bool>;
        enable_respawn_screen: Primitive<bool>;
        dimension_type: String;
        dimension_name: String;
        hashed_seed: Primitive<i64>;
        gamemde: Primitive<u8>;
        previous_gamemode: Primitive<i8>;
        is_debug: Primitive<bool>;
        is_flat: Primitive<bool>;
        death_info: BoolOption<JoinGameDeathInfo>;
        portal_cooldown: VarInt;
    }

    JoinGameDeathInfo {
        dimension_name: String;
        position: Primitive<u64>;
    }
}
