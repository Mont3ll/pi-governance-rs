mod memory;
mod recall;
mod relationship;
mod store;

pub use memory::*;
pub use recall::*;
pub use relationship::*;
pub use store::*;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityRecommendation {
    pub id: String,
    pub summary: String,
    pub reason: String,
    pub review_required: bool,
    pub mutation_performed: bool,
}
