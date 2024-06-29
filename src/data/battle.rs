use std::{
    cmp::{max, min},
    fmt::{Debug, Formatter}
};
use eyre::Result;
use thiserror::Error;

pub struct LivingBuilder {
    living: Living
}

#[derive(Error, Debug)]
pub enum LivingBuilderError {
    #[error("life: health > max_health")]
    HealthGreaterThanMaxHealth
}

impl LivingBuilder {
    pub fn new() -> Self {
        Self {
            living: Living {
                health: 0,
                max_health: 0
            }
        }
    }

    pub fn health(mut self, amount: u32) -> Self {
        self.living.health = amount;
        self.living.max_health = amount;
        self
    }

    pub fn max_health(mut self, amount: u32) -> Self {
        self.living.max_health = amount;
        self
    }

    pub fn build(self) -> Result<Living, LivingBuilderError> {
        let life = self.living;

        if life.health > life.max_health {
            return Err(LivingBuilderError::HealthGreaterThanMaxHealth)
        }

        Ok(life)
    }
}

#[derive(Clone)]
pub struct Living {
    health: u32,
    max_health: u32
}

impl Debug for Living {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(
            &format!("Living(hp: {}/{})", self.health, self.max_health)
        )
    }
}

impl Living {
    pub fn new() -> LivingBuilder {
        LivingBuilder::new()
    }

    pub fn heal(&mut self, amount: u32) {
        self.health = min(
            self.max_health,
            self.health + amount
        );
    }

    pub fn damage(&mut self, amount: u32) {
        self.health = max(
            0,
            self.health - amount
        );
    }

    pub fn health(&self) -> u32 {
        self.health
    }

    pub fn max_health(&self) -> u32 {
        self.max_health
    }

    pub fn dead(&self) -> bool {
        self.health == 0
    }
}