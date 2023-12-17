use std::marker::PhantomData;

use serde::{
    ser::{Error, SerializeMap, SerializeSeq, SerializeStruct, SerializeTupleStruct},
    Serialize,
};

#[repr(transparent)]
#[derive(Debug)]
pub struct SerializerAdapter<S, TOk, TError> {
    serializer: S,
    _phantom_ok: PhantomData<TOk>,
    _phantom_err: PhantomData<TError>,
}

impl<S, TOk, TError> SerializerAdapter<S, TOk, TError> {
    pub fn new(serializer: S) -> Self {
        Self {
            serializer,
            _phantom_ok: PhantomData,
            _phantom_err: PhantomData,
        }
    }
}

impl<S: SerializeMap, TOk: From<S::Ok>, TError: Error + From<S::Error>> SerializeMap
    for SerializerAdapter<S, TOk, TError>
{
    type Ok = TOk;
    type Error = TError;

    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        self.serializer.serialize_key(key).map_err(TError::from)
    }

    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        self.serializer.serialize_value(value).map_err(TError::from)
    }

    fn serialize_entry<K: ?Sized + Serialize, V: ?Sized + Serialize>(
        &mut self,
        key: &K,
        value: &V,
    ) -> Result<(), Self::Error> {
        self.serializer
            .serialize_entry(key, value)
            .map_err(TError::from)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.serializer.end().map(TOk::from).map_err(TError::from)
    }
}

impl<S: SerializeStruct, TOk: From<S::Ok>, TError: Error + From<S::Error>> SerializeStruct
    for SerializerAdapter<S, TOk, TError>
{
    type Ok = TOk;
    type Error = TError;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error> {
        self.serializer
            .serialize_field(key, value)
            .map_err(TError::from)
    }

    fn skip_field(&mut self, key: &'static str) -> Result<(), Self::Error> {
        self.serializer.skip_field(key).map_err(TError::from)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.serializer.end().map(TOk::from).map_err(TError::from)
    }
}

impl<S: SerializeSeq, TOk: From<S::Ok>, TError: Error + From<S::Error>> SerializeSeq
    for SerializerAdapter<S, TOk, TError>
{
    type Ok = TOk;
    type Error = TError;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        self.serializer
            .serialize_element(value)
            .map_err(TError::from)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.serializer.end().map(TOk::from).map_err(TError::from)
    }
}

/// Adapter for treating [`SerializeTuple`] as [`SerializeSeq`].
#[repr(transparent)]
#[derive(Debug)]
pub struct SerializeSeqAsSerializeTupleStruct<S, TOk, TError> {
    serializer: S,
    _phantom_ok: PhantomData<TOk>,
    _phantom_err: PhantomData<TError>,
}

impl<S, TOk, TError> SerializeSeqAsSerializeTupleStruct<S, TOk, TError> {
    pub const fn new(serializer: S) -> Self {
        Self {
            serializer,
            _phantom_ok: PhantomData,
            _phantom_err: PhantomData,
        }
    }
}

impl<S: SerializeSeq, TOk: From<S::Ok>, TError: Error + From<S::Error>> SerializeTupleStruct
    for SerializeSeqAsSerializeTupleStruct<S, TOk, TError>
{
    type Ok = TOk;
    type Error = TError;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        self.serializer
            .serialize_element(value)
            .map_err(TError::from)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.serializer.end().map(TOk::from).map_err(TError::from)
    }
}
