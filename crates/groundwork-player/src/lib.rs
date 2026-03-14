//! Embodied player agent framework for GROUNDWORK.
//!
//! This crate provides deterministic scenario execution, trace recording,
//! and programmatic evaluation for automated playtesting.
//!
//! # Architecture
//!
//! - **Action**: what the player does (place seed, tick, inspect, etc.)
//! - **Observation**: what the player sees back (CLI-like text output)
//! - **OracleSnapshot**: privileged sim state for evaluators
//! - **Trace**: complete record of a run (action → observation → oracle triples)
//! - **Evaluator**: judges a trace and produces verdicts
//! - **Scenario**: scripted action sequence + evaluators
//! - **Planner**: autonomous decision-maker (scripted or LLM-powered)
//!
//! The actor (scenario or planner) only sees Observations.
//! Evaluators use OracleSnapshots for ground truth scoring.

pub mod action;
pub mod evaluator;
pub mod observer;
pub mod oracle;
pub mod planner;
pub mod planner_scripted;
#[cfg(feature = "llm")]
pub mod planner_llm;
pub mod runner;
pub mod scenario;
pub mod scenarios;
pub mod trace;
