//! Privacy-preserving analysis components
//! 
//! Ensures all analysis happens locally with zero external calls
//! for complete privacy protection of user behavioral data.

pub mod local_inference;

pub use local_inference::{LocalInferenceEngine, NetworkIsolationReport};