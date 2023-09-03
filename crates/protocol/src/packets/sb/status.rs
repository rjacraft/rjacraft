use crate::packets::prelude::*;

packets_struct! {
    Request {}

    Ping {
        payload: Primitive<i64>;
    }
}
