// Use Candid for on-chain data
use candid::{CandidType, Deserialize as CandidDeserialize, Principal};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use ic_stable_structures::Storable;
use ic_stable_structures::storable::Bound;
use std::borrow::Cow;

use crate::MODELS;

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
pub struct ClassifierModelData {
    pub(crate) data_points: Vec<DataPoint>,
    pub(crate) metrics: Metrics,
    pub(crate) metrics_history: Vec<Metrics>,
}

#[derive(CandidType, CandidDeserialize, Clone, Debug, Serialize)]
pub struct LLMModelData {
    pub(crate) hugging_face_url: String,
    pub(crate) cat_metrics: Option<ContextAssociationTestMetricsBag>,
    pub(crate) cat_metrics_history: Vec<ContextAssociationTestMetricsBag>,
}

#[derive(CandidType, CandidDeserialize, Clone, Debug)]
pub struct Model {
    pub(crate) model_id: u128,
    pub(crate) model_name: String,
    pub(crate) owners: Vec<Principal>,
    pub(crate) details: ModelDetails,
    pub(crate) model_type: ModelType
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
    pub(crate) llm_models: Vec<u128>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct HuggingFaceResponseItem {
    pub(crate) generated_text: Option<String>,
}

#[derive(Clone, Debug, CandidType, CandidDeserialize)]
pub enum ModelType {
    Classifier(ClassifierModelData),
    LLM(LLMModelData),
}

// Useful function that panics in the case that the model is NOT a classifier
pub fn get_classifier_model_data(model: &Model) -> ClassifierModelData {
    match model.model_type {
        ModelType::Classifier(ref model_data) => model_data.clone(),
        _ => panic!("A classifier model was expected, got another type of model instead"),
    }
}

// Useful function that panics in the case that the model is NOT a classifier
pub fn get_llm_model_data(model: &Model) -> LLMModelData {
    match model.model_type {
        ModelType::LLM(ref model_data) => model_data.clone(),
        _ => panic!("A classifier model was expected, got another type of model instead"),
    }
}

// LLMs
#[derive(Serialize, Copy, Clone, PartialEq, Debug, CandidType, CandidDeserialize)]
pub enum ContextAssociationTestResult {
    Stereotype,
    AntiStereotype,
    Neutral,
    Other,
}

#[derive(Serialize, Deserialize, CandidType, Clone, Debug)]
pub struct ContextAssociationTestMetrics {
    pub(crate) stereotype: u32,
    pub(crate) anti_stereotype: u32,
    pub(crate) neutral: u32,
    pub(crate) other: u32,
}

#[derive(Serialize, Deserialize, CandidType, Clone, Debug)]
pub struct ContextAssociationTestMetricsBag {
    pub(crate) general: ContextAssociationTestMetrics,
    pub(crate) intersentence: ContextAssociationTestMetrics,
    pub(crate) intrasentence: ContextAssociationTestMetrics,
    pub(crate) gender: ContextAssociationTestMetrics,
    pub(crate) race: ContextAssociationTestMetrics,
    pub(crate) religion: ContextAssociationTestMetrics,
    pub(crate) profession: ContextAssociationTestMetrics,
    pub(crate) error_count: u32,
    pub(crate) timestamp: u64,
    pub(crate) intrasentence_prompt_template: String,
    pub(crate) intersentence_prompt_template: String,
    pub(crate) seed: u32,
    // precalculated fields
    pub(crate) icat_score_intra: f32,
    pub(crate) icat_score_inter: f32,
    pub(crate) icat_score_gender: f32,
    pub(crate) icat_score_race: f32,
    pub(crate) icat_score_profession: f32,
    pub(crate) icat_score_religion: f32,
    pub(crate) general_lms: f32,
    pub(crate) general_ss: f32,
    pub(crate) general_n: u32,
    pub(crate) icat_score_general: f32,
    pub(crate) data_points: Vec<ContextAssociationTestDataPoint>,
}

#[derive(Serialize, CandidType, CandidDeserialize, Clone, Debug)]
pub enum ContextAssociationTestType {
    Intrasentence,
    Intersentence,
}

#[derive(Serialize, CandidType, CandidDeserialize, Clone, Debug)]
pub struct ContextAssociationTestDataPoint {
    pub(crate) data_point_id: u128,
    pub(crate) prompt: String,
    pub(crate) answer: Option<String>,
    pub(crate) result: Option<ContextAssociationTestResult>,
    pub(crate) error: bool,
    pub(crate) test_type: ContextAssociationTestType,
    pub(crate) timestamp: u64,
}

impl Storable for ClassifierModelData {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(candid::encode_one(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        candid::decode_one(&bytes).unwrap()
    }

    const BOUND: Bound = Bound::Unbounded;
}


impl Storable for LLMModelData {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(candid::encode_one(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        candid::decode_one(&bytes).unwrap()
    }
    
    const BOUND: Bound = Bound::Unbounded;
}
#[derive(Serialize, Debug, CandidType, CandidDeserialize, Clone)]
pub struct ContextAssociationTestAPIResult {
    pub error_count: u32,
    pub general_ss: f32, 
    pub general_n: u32,
    pub general_lms: f32,
    pub general: ContextAssociationTestMetrics,
    pub icat_score_general: f32,
    pub icat_score_gender: f32,
    pub icat_score_religion: f32,
    pub icat_score_profession: f32,
    pub icat_score_race: f32,
    pub icat_score_intra: f32,
    pub icat_score_inter: f32,
}
