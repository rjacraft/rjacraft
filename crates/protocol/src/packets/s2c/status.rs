use crate::packets::prelude::*;

packets_struct! {
    Response {
        response: JsonString<{ 1 << 18 }, ServerStatus>;
    }

    Pong {
        payload: Primitive<i64>;
    }
}
