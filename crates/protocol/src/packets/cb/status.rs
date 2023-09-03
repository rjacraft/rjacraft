use crate::packets::prelude::*;

packets_struct! {
    Response {
        response: JsonString<serde_json::Value>; // todo struct
    }

    Pong {
        payload: Primitive<i64>;
    }
}
