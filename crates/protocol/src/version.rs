const SNAPSHOT_FLAG: i32 = 1 << 30;

/// A protocol version that can be encoded in the numeric format.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProtocolVersion {
    Stable(i32),
    Snapshot(i32),
}

impl From<ProtocolVersion> for i32 {
    fn from(value: ProtocolVersion) -> Self {
        match value {
            ProtocolVersion::Stable(x) => x,
            ProtocolVersion::Snapshot(x) => x + SNAPSHOT_FLAG,
        }
    }
}

impl From<i32> for ProtocolVersion {
    fn from(value: i32) -> Self {
        if value >= SNAPSHOT_FLAG {
            ProtocolVersion::Snapshot(value - SNAPSHOT_FLAG)
        } else {
            ProtocolVersion::Stable(value)
        }
    }
}

impl<'de> serde::Deserialize<'de> for ProtocolVersion {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(i32::deserialize(deserializer)?.into())
    }
}

impl serde::Serialize for ProtocolVersion {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        i32::serialize(&(*self).into(), serializer)
    }
}
