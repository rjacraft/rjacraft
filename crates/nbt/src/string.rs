use std::{borrow::Cow, io, ops::Deref};

use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// A string which can safely be stored in a NBT.
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct NbtStr<'a>(Cow<'a, [u8]>);

/// An error which may occur while creating a [`NbtStr`] from `&str`.
#[derive(Debug, thiserror::Error)]
#[error("NBT string length cannot exceed {} but is {0}", u16::MAX)]
pub struct NbtStrFromStrError(usize);

impl<'a> NbtStr<'a> {
    pub fn empty() -> Self {
        // empty `Vec`s don't require allocations, so we can start with an owned variant
        Self(Cow::Owned(Vec::new()))
    }

    /// Converts a vector of bytes to a `NbtStr`
    /// without checking that the string contains valid Java CESU-8.
    ///
    /// Note that the length is still checked.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it does not check
    /// that the bytes passed to it are valid Java CESU-8.
    pub unsafe fn from_cesu8_unchecked(bytes: Vec<u8>) -> Result<Self, NbtStrFromStrError> {
        debug_assert!(
            cesu8::is_valid_java_cesu8(
                std::str::from_utf8(&bytes).expect("the string is not a valid UTF-8 string"),
            ),
            "the string is not a valid Java CESU-8 String",
        );

        if u16::try_from(bytes.len()).is_ok() {
            Ok(Self(Cow::Owned(bytes)))
        } else {
            Err(NbtStrFromStrError(bytes.len()))
        }
    }

    /// Returns the length of this String, in bytes, not chars or graphemes.
    ///
    /// # Example
    ///
    /// ```
    /// use rjacraft_nbt::string::NbtStr;
    ///
    /// let text = NbtStr::try_from("Hello").unwrap();
    /// assert_eq!(text.len(), 5);
    /// ```
    pub fn len(&self) -> u16 {
        // the conversion is always valid since the length should be checked on creation
        self.0.len() as u16
    }

    /// Returns `true` if this string has a length of zero, and `false` otherwise.
    ///
    /// # Example
    ///
    /// ```
    /// use rjacraft_nbt::string::NbtStr;
    ///
    /// let text = NbtStr::try_from("Hello").unwrap();
    /// assert!(!text.is_empty());
    /// let text = NbtStr::try_from("").unwrap();
    /// assert!(text.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn into_inner(self) -> Cow<'a, [u8]> {
        self.0
    }

    pub fn as_string(&'a self) -> Cow<'a, str> {
        let string = cesu8::from_java_cesu8(self.0.as_ref());
        #[cfg(debug_assertions)]
        {
            string.expect("string should be validated on creation")
        }
        #[cfg(not(debug_assertions))]
        {
            // SAFETY: this type guarantees that the internal string is a valid Java CESU-8 string
            unsafe { string.unwrap_unchecked() }
        }
    }

    pub fn write(&self, mut write: impl io::Write) -> io::Result<()> {
        write.write_all(&self.len().to_be_bytes())?;
        write.write_all(&self.0)?;
        Ok(())
    }

    #[cfg(feature = "async")]
    pub async fn write_async<W: tokio::io::AsyncWrite + Unpin>(
        self,
        write: &mut W,
    ) -> io::Result<()> {
        use tokio::io::AsyncWriteExt;

        write.write_u16(self.len()).await?;
        write.write_all(&self.0).await?;
        Ok(())
    }
}

impl<'a> Deref for NbtStr<'a> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

impl AsRef<[u8]> for NbtStr<'_> {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl<'a> TryFrom<&'a str> for NbtStr<'a> {
    type Error = NbtStrFromStrError;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        let value = cesu8::to_java_cesu8(value);
        if u16::try_from(value.len()).is_ok() {
            Ok(Self(value))
        } else {
            Err(NbtStrFromStrError(value.len()))
        }
    }
}

impl<'a> TryFrom<String> for NbtStr<'a> {
    type Error = NbtStrFromStrError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let bytes = match cesu8::to_java_cesu8(&value) {
            Cow::Borrowed(_) => {
                // since we did not have to do a copy, the original string was kept unchanged,
                // thus we can reuse it
                value.into_bytes()
            }
            Cow::Owned(bytes) => {
                // we have a fresh copy, now we only have to do the size check
                bytes
            }
        };

        // SAFETY: we just checked that the string is a valid Java CESU-8 string
        unsafe { Self::from_cesu8_unchecked(bytes) }
    }
}

impl<'a> Serialize for NbtStr<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_string().as_ref())
    }
}

impl<'de> Deserialize<'de> for NbtStr<'de> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        use serde::de::Error;

        let string = Cow::<'de, str>::deserialize(deserializer)?;

        match string {
            Cow::Borrowed(string) => Self::try_from(string),
            Cow::Owned(string) => Self::try_from(string),
        }
        .map_err(|e| Error::custom(e.to_string()))
    }
}
