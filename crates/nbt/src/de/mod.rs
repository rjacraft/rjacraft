mod map;
mod payload;
mod seq;
mod adapter;

pub struct RootDeserializer<R> {
    pub reader: R,
}
