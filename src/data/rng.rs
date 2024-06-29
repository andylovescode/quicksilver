use rand::random;
use serde::{Deserialize, Serialize};
use std::{
    fmt::{Debug, Formatter},
    ops::Range,
};

#[derive(Serialize, Deserialize, Clone)]
pub struct Random(u32);

impl Default for Random {
    fn default() -> Self {
        Self::new()
    }
}

impl Random {
    pub fn new() -> Self {
        Self(random())
    }

    pub fn get(&self, range: Range<f32>) -> f32 {
        range.start + self.0 as f32 / i32::MAX as f32 * (range.end - range.start)
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Chance(Random);

impl Debug for Chance {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("Chance({})", self.threshold()))
    }
}

impl Chance {
    pub fn new() -> Self {
        Self(Random::new())
    }

    pub fn threshold(&self) -> f32 {
        self.0.get(0f32..1f32)
    }

    pub fn eval(&self, chance: f32) -> bool {
        self.threshold() <= chance
    }
}

impl Default for Chance {
    fn default() -> Self {
        Self::new()
    }
}
