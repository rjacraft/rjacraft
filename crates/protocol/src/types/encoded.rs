//! A protocol value that's been encoded in advance. Useful for avoiding unnecessary encoding runs.

use std::marker::PhantomData;

use bytes::{Buf, BufMut};

use crate::ProtocolType;

/// Please clone this as much as you want!
#[derive(Debug)]
pub struct Encoded<T>(bytes::Bytes, PhantomData<T>);

impl<T: ProtocolType> Encoded<T> {
    pub fn new(value: &T) -> Result<Self, T::EncodeError> {
        let mut bytes = bytes::BytesMut::new();

        value.encode(&mut bytes)?;

        Ok(Self(bytes.freeze(), PhantomData))
    }

    pub fn data(&self) -> bytes::Bytes {
        self.0.clone()
    }
}

impl<T: ProtocolType> ProtocolType for Encoded<T> {
    type DecodeError = T::DecodeError;
    type EncodeError = T::EncodeError;

    fn decode(_buffer: &mut impl Buf) -> Result<Self, Self::DecodeError> {
        todo!()
    }

    fn encode(&self, buffer: &mut impl BufMut) -> Result<(), Self::EncodeError> {
        buffer.put(self.0.chunk());

        Ok(())
    }
}

impl<T> Clone for Encoded<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone(), PhantomData)
    }
}
