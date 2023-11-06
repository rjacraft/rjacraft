macro_rules! impl_tuple {
    ($($T:ident),*) => {
        #[allow(non_snake_case)]
        const _: () = {
            #[derive(Debug, thiserror::Error)]
            pub enum DecodeError<$($T: std::error::Error),*> {
                $(
                    #[error(transparent)]
                    $T($T),
                )*
            }

            #[derive(Debug, thiserror::Error)]
            pub enum EncodeError<$($T: std::error::Error),*> {
                $(
                    #[error(transparent)]
                    $T($T),
                )*
            }

            impl<$($T),*> crate::ProtocolType for ($($T),*) where $($T: crate::ProtocolType,)* {
                type DecodeError = DecodeError<$(<$T as crate::ProtocolType>::DecodeError),*>;
                type EncodeError = EncodeError<$(<$T as crate::ProtocolType>::EncodeError),*>;

                fn decode(buffer: &mut impl bytes::Buf) -> Result<Self, Self::DecodeError> {
                    Ok(($(<$T as crate::ProtocolType>::decode(buffer).map_err(DecodeError::$T)?),*))
                }

                fn encode(&self, buffer: &mut impl bytes::BufMut) -> Result<(), Self::EncodeError> {
                    let ($($T),*) = self;

                    $(<$T as crate::ProtocolType>::encode($T, buffer).map_err(EncodeError::$T)?;)*

                    Ok(())
                }
            }
        };
    };
}

bevy_ecs::all_tuples!(impl_tuple, 2, 15, T);
