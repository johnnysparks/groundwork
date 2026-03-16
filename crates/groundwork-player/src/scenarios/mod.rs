//! Built-in scenarios for automated playtesting.
//!
//! Each module defines one or more scenarios that test specific aspects
//! of the game. Scenarios are deterministic and produce traces that
//! can be evaluated programmatically.

pub mod autonomous;
pub mod basic_growth;
pub mod camera_exploration;
pub mod ecosystem;
pub mod gameplay;
pub mod water_system;
