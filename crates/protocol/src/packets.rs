//! Hand-written packet types

pub mod cb;
pub mod sb;

mod macros;

mod prelude {
    pub(crate) use super::macros::*;
    pub use crate::types::*;
}
