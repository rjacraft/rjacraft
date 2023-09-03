const SNAPSHOT_FLAG: i32 = 1 << 30;

/// A protocol version that can be encoded in the numeric format.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProtocolVersion {
    Stable(i32),
    Snapshot(i32),
}

impl Into<i32> for ProtocolVersion {
    fn into(self) -> i32 {
        match self {
            Self::Stable(x) => x,
            Self::Snapshot(x) => x + SNAPSHOT_FLAG,
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
