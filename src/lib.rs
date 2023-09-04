//! Basic components for Rjacraft server development.
//!
//! # Features
#![doc = document_features::document_features!()]
//!

// This crate combines higher-level abstractions from other crates so `unsafe` is a red flag here
#![deny(unsafe_code)]

#[cfg(feature = "core")]
pub use rjacraft_core as core;
#[cfg(feature = "network")]
pub use rjacraft_network as network;
#[cfg(feature = "protocol")]
pub use rjacraft_protocol as protocol;
