use std::collections::BTreeMap;

use crate::{string::NbtStr, Nbt};

/// [NBT Compound](https://wiki.vg/NBT#Specification:compound_tag) value.
#[derive(Debug, PartialEq, Clone)] // FIXME: `Serialize`
pub struct NbtCompound<'a>(BTreeMap<NbtStr<'a>, Nbt<'a>>);
