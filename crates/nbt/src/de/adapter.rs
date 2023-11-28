use std::marker::PhantomData;

use serde::{
    de::{DeserializeSeed, Error, MapAccess, SeqAccess},
    Deserialize,
};

#[repr(transparent)]
#[derive(Debug)]
pub struct DeserializerAdapter<S, TError> {
    serializer: S,
    _phantom_err: PhantomData<TError>,
}

impl<'de, S: SeqAccess<'de>, TError: Error + From<S::Error>> SeqAccess<'de>
    for DeserializerAdapter<S, TError>
{
    type Error = TError;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        self.serializer
            .next_element_seed(seed)
            .map_err(TError::from)
    }

    fn next_element<T>(&mut self) -> Result<Option<T>, Self::Error>
    where
        T: Deserialize<'de>,
    {
        self.serializer.next_element().map_err(TError::from)
    }

    fn size_hint(&self) -> Option<usize> {
        self.serializer.size_hint()
    }
}

impl<'de, S: MapAccess<'de>, TError: Error + From<S::Error>> MapAccess<'de>
    for DeserializerAdapter<S, TError>
{
    type Error = TError;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: DeserializeSeed<'de>,
    {
        self.serializer.next_key_seed(seed).map_err(TError::from)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        self.serializer.next_value_seed(seed).map_err(TError::from)
    }

    fn next_entry_seed<K, V>(
        &mut self,
        kseed: K,
        vseed: V,
    ) -> Result<Option<(K::Value, V::Value)>, Self::Error>
    where
        K: DeserializeSeed<'de>,
        V: DeserializeSeed<'de>,
    {
        self.serializer
            .next_entry_seed(kseed, vseed)
            .map_err(TError::from)
    }

    fn next_key<K>(&mut self) -> Result<Option<K>, Self::Error>
    where
        K: Deserialize<'de>,
    {
        self.serializer.next_key().map_err(TError::from)
    }

    fn next_value<V>(&mut self) -> Result<V, Self::Error>
    where
        V: Deserialize<'de>,
    {
        self.serializer.next_value().map_err(TError::from)
    }

    fn next_entry<K, V>(&mut self) -> Result<Option<(K, V)>, Self::Error>
    where
        K: Deserialize<'de>,
        V: Deserialize<'de>,
    {
        self.serializer.next_entry().map_err(TError::from)
    }

    fn size_hint(&self) -> Option<usize> {
        self.serializer.size_hint()
    }
}
