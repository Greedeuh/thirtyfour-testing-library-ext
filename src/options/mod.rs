//! Testing Library options module
//!
//! This module contains all option types for testing-library queries,
//! organized into a clean hierarchy.

/// Common trait for all testing library options
pub mod common;
/// Label text query options
pub mod label_text;
/// Role-based query options
pub mod role;
/// Simple options for basic query types
pub mod simple;

// Re-export everything for convenience
pub use common::{process_raw_javascript_markers, RawJavaScript, TestingLibraryOptions, TextMatch};
pub use label_text::*;
pub use role::*;
pub use simple::*;
