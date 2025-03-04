// Use Candid for on-chain data
use candid::{CandidType, Deserialize as CandidDeserialize, Principal};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use ic_stable_structures::Storable;
use ic_stable_structures::storable::Bound;
use std::borrow::Cow;

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
pub(crate) struct CachedThresholds {
    pub(crate) thresholds: Option<HashMap<String, (f64, bool)>>
}

#[derive(CandidType, CandidDeserialize, Clone, Debug)]
pub struct Model {
    pub(crate) model_id: u128,
    pub(crate) model_name: String,
    pub(crate) owners: Vec<Principal>,
    pub(crate) data_points: Vec<DataPoint>,
    pub(crate) metrics: Metrics,
    pub(crate) details: ModelDetails,
    pub(crate) metrics_history: Vec<Metrics>,
    pub(crate) cached_thresholds: Option<CachedThresholds>,
}

impl Storable for Model {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(candid::encode_one(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        candid::decode_one(&bytes).unwrap()
    }

    const BOUND: Bound = Bound::Unbounded;
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
pub struct User {
    pub(crate) user_id: Principal,
    pub(crate) models: Vec<u128>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct HuggingFaceResponseItem {
    pub(crate) generated_text: Option<String>,
}
