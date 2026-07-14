mod memory;
mod relationship;

pub use memory::*;
pub use relationship::*;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityRecommendation { pub id: String, pub summary: String, pub reason: String, pub review_required: bool, pub mutation_performed: bool }
