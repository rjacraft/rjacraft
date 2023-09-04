use core::fmt;

const SNAPSHOT_FLAG: i32 = 1 << 30;

/// A protocol version number with convenient analysis methods.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProtocolVersion(pub i32);

impl ProtocolVersion {
    /// Numbers bigger than 2^30 will not produce valid version numbers, so just please don't hit
    /// that mark.
    pub const fn from_release(number: i32) -> Self {
        Self(number)
    }

    /// Numbers bigger than 2^30 will not produce valid version numbers, so just please don't hit
    /// that mark.
    pub const fn from_snapshot(number: i32) -> Self {
        Self(number + SNAPSHOT_FLAG)
    }

    pub fn version(self) -> i32 {
        self.0 % SNAPSHOT_FLAG
    }

    pub fn is_snapshot(self) -> bool {
        self.0 > SNAPSHOT_FLAG
    }
}

impl fmt::Display for ProtocolVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_snapshot() {
            write!(f, "Snapshot protocool {}", self.version())
        } else {
            write!(f, "Release protocol {}", self.version())
        }
    }
}

impl<'de> serde::Deserialize<'de> for ProtocolVersion {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(Self(i32::deserialize(deserializer)?))
    }
}

impl serde::Serialize for ProtocolVersion {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        i32::serialize(&self.0, serializer)
    }
}
