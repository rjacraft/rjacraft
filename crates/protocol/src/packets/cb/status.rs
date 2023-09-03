use crate::packets::prelude::*;

packets_struct! {
    Response {
        response: JsonString<StatusObject>; // todo struct
    }

    Pong {
        payload: Primitive<i64>;
    }
}
