use crate::packets::prelude::*;

packets_struct! {
    Response {
        response: JsonString<{ 1 << 18 }, StatusObject>;
    }

    Pong {
        payload: Primitive<i64>;
    }
}
