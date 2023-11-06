use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Bounds<T> {
    pub min_inclusive: T,
    pub max_inclusive: T,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Clamped<P, T> {
    #[serde(flatten)]
    pub bounds: Bounds<T>,
    pub source: Box<P>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ClampedNormal<T> {
    pub mean: f32,
    pub deviation: f32,
    #[serde(flatten)]
    pub bounds: Bounds<T>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Weighted<P> {
    data: P,
    weight: i32,
}

/// This doesn't deserialize very well. And most of the variants have not been tested even for
/// serialization.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum IntProvider {
    #[serde(rename = "minecraft:constant")]
    Contant { value: i32 },
    #[serde(rename = "minecraft:uniform")]
    Uniform { value: Bounds<i32> },
    #[serde(rename = "minecraft:uniform")]
    BiasedToBottom { value: Bounds<i32> },
    #[serde(rename = "minecraft:clamped")]
    Clamped { value: Clamped<IntProvider, i32> },
    #[serde(rename = "minecraft:clamped_normal")]
    ClampedNormal { value: ClampedNormal<i32> },
    #[serde(rename = "minecraft:clamped_normal")]
    WeightedList {
        distribution: Vec<Weighted<IntProvider>>,
    },
}
