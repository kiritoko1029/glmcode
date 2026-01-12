pub mod context_window;
pub mod decode_speed;
pub mod directory;
pub mod git;
pub mod glm_usage;
pub mod model;
pub mod output_style;
pub mod session;
pub mod update;
pub mod usage;

use crate::config::{InputData, SegmentId};
use std::collections::HashMap;

// New Segment trait for data collection only
pub trait Segment {
    fn collect(&self, input: &InputData) -> Option<SegmentData>;
    fn id(&self) -> SegmentId;
}

#[derive(Debug, Clone)]
pub struct SegmentData {
    pub primary: String,
    pub secondary: String,
    pub metadata: HashMap<String, String>,
}

// Re-export all segment types
pub use context_window::ContextWindowSegment;
pub use decode_speed::DecodeSpeedSegment;
pub use directory::DirectorySegment;
pub use git::GitSegment;
pub use glm_usage::GlmUsageSegment;
pub use model::ModelSegment;
pub use output_style::OutputStyleSegment;
pub use session::SessionSegment;
pub use update::UpdateSegment;
pub use usage::UsageSegment;
