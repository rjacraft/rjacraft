//! Hand-written packet types

pub mod c2s;
pub mod s2c;

mod macros;

mod prelude {
    pub(crate) use super::macros::*;
    pub use crate::types::*;
}
