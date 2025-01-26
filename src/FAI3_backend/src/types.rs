// Use Candid for on-chain data
use candid::{CandidType, Deserialize as CandidDeserialize, Principal};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(CandidType, CandidDeserialize, Clone, Debug)]
pub struct DataPoint {
    pub(crate) data_point_id: u128,
    pub(crate) target: bool,
    pub(crate) privileged_map: HashMap<String, u128>,
    pub(crate) predicted: bool,
    pub(crate) features: Vec<f64>,
    pub(crate) timestamp: u64,
}

#[derive(CandidType, CandidDeserialize, Clone, Debug)]
pub struct AverageMetrics {
    pub statistical_parity_difference: Option<f32>,
    pub(crate) disparate_impact: Option<f32>,
    pub average_odds_difference: Option<f32>,
    pub(crate) equal_opportunity_difference: Option<f32>,
}

#[derive(CandidType, CandidDeserialize, Clone, Debug)]
pub struct PrivilegedIndex {
    pub(crate) variable_name: String,
    pub(crate) value: f32,
}


#[derive(CandidType, CandidDeserialize, Clone, Debug)]
pub struct Metrics {
    pub(crate) statistical_parity_difference: Option<Vec<PrivilegedIndex>>,
    pub(crate) disparate_impact: Option<Vec<PrivilegedIndex>>,
    pub(crate) average_odds_difference: Option<Vec<PrivilegedIndex>>,
    pub(crate) equal_opportunity_difference: Option<Vec<PrivilegedIndex>>,
    pub(crate) average_metrics: AverageMetrics,
    pub(crate) accuracy: Option<f32>,
    pub(crate) precision: Option<f32>,
    pub(crate) recall: Option<f32>,
    pub(crate) timestamp: u64,
}

#[derive(CandidType, CandidDeserialize, Clone, Debug)]
pub struct Model {
    pub(crate) model_id: u128,
    pub(crate) model_name: String,
    pub(crate) user_id: Principal,
    pub(crate) data_points: Vec<DataPoint>,
    pub(crate) metrics: Metrics,
    pub(crate) details: ModelDetails,
    pub(crate) metrics_history: Vec<Metrics>,
}

#[derive(CandidType, CandidDeserialize, Clone, Debug)]
pub struct ModelDetails {
    pub(crate) description: String,
    pub(crate) framework: String,
    pub(crate) version: String,
    pub(crate) objective: String,
    pub(crate) url: String,
}

#[derive(CandidType, CandidDeserialize, Clone, Debug)]
// #[derive(CandidType, CandidDeserialize, Clone, Debug)]
pub struct User {
    pub(crate) user_id: Principal,
    pub(crate) models: HashMap<u128, Model>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct HuggingFaceResponseItem {
    pub(crate) generated_text: Option<String>,
}
