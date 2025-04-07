// Use Candid for on-chain data
use candid::{CandidType, Deserialize as CandidDeserialize, Principal};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use ic_stable_structures::Storable;
use ic_stable_structures::storable::Bound;
use std::borrow::Cow;

pub type PrivilegedMap = HashMap<String, u128>;

#[derive(CandidType, CandidDeserialize, Clone, Debug, PartialEq)]
pub struct DataPoint {
    pub data_point_id: u128,
    pub target: bool,
    pub privileged_map: PrivilegedMap,
    pub predicted: bool,
    pub features: Vec<f64>,
    pub timestamp: u64,
}

#[derive(CandidType, CandidDeserialize, Clone, Debug, PartialEq)]
pub struct LLMDataPointCounterFactual {
    pub prompt: Option<String>,
    pub response: Option<String>,
    pub valid: bool,
    pub error: bool,
    pub target: bool,
    pub timestamp: u64,
    pub predicted: Option<bool>,
    pub features: Vec<f64>,
}

// Represents a data point for using LLMs as classifiers
// So the same classifier metrics can be calculated over this
#[derive(CandidType, CandidDeserialize, Clone, Debug, PartialEq)]
pub struct LLMDataPoint {
    pub data_point_id: u128,
    pub target: bool,
    pub predicted: Option<bool>,
    pub features: Vec<f64>,
    pub timestamp: u64,
    pub prompt: String,
    pub response: Option<String>,
    pub valid: bool,
    pub error: bool,
    pub counter_factual: Option<LLMDataPointCounterFactual>,
}

impl LLMDataPoint { 
    /// Transforms a LLM_DataPoint to a DataPoint, so it can be used for metrics
    // If the LLM DataPoint had an error of some type, it returns None
    pub fn to_data_point(&self, privileged_map: PrivilegedMap) -> Option<DataPoint> {
        match self.predicted {
            Some(pred) => Some(DataPoint {
                data_point_id: self.data_point_id,
                target: self.target,
                privileged_map,
                predicted: pred,
                features: self.features.clone(),
                timestamp: self.timestamp,
            }),
            None => None,
        }
    }

    pub fn reduce_to_data_points(data_points: &Vec<LLMDataPoint>, privileged_map: PrivilegedMap) -> Vec<DataPoint> {
        data_points.into_iter().filter_map(|dp| dp.to_data_point(privileged_map.clone())).collect()
    }
}

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq)]
pub struct KeyValuePair {
    pub key: String,
    pub value: u128,
}

#[derive(CandidType, CandidDeserialize, Clone, Debug, PartialEq)]
pub struct CounterFactualModelEvaluationResult {
    pub change_rate_overall: f32,
    pub change_rate_sensible_attributes: Vec<f32>,
    pub total_sensible_attributes: Vec<u32>,
    pub sensible_attribute: String,
}

#[derive(CandidType, CandidDeserialize, Clone, Debug, PartialEq)]
pub struct ModelEvaluationResult {
    pub model_evaluation_id: u128,
    pub dataset: String,
    pub timestamp: u64,
    pub metrics: Metrics,
    pub privileged_map: Vec<KeyValuePair>,
    // data_points is to be used in the future,
    // To replace the metrics and metrics_history
    pub data_points: Option<Vec<DataPoint>>,
    pub llm_data_points: Option<Vec<LLMDataPoint>>,
    pub prompt_template: Option<String>,
    pub counter_factual: Option<CounterFactualModelEvaluationResult>,
}

#[derive(CandidType, CandidDeserialize, Clone, Debug, PartialEq)]
pub struct AverageMetrics {
    pub statistical_parity_difference: Option<f32>,
    pub disparate_impact: Option<f32>,
    pub average_odds_difference: Option<f32>,
    pub equal_opportunity_difference: Option<f32>,
}

#[derive(CandidType, CandidDeserialize, Clone, Debug, PartialEq)]
pub struct PrivilegedIndex {
    pub variable_name: String,
    pub value: f32,
}


#[derive(CandidType, CandidDeserialize, Clone, Debug, PartialEq)]
pub struct Metrics {
    pub statistical_parity_difference: Option<Vec<PrivilegedIndex>>,
    pub disparate_impact: Option<Vec<PrivilegedIndex>>,
    pub average_odds_difference: Option<Vec<PrivilegedIndex>>,
    pub equal_opportunity_difference: Option<Vec<PrivilegedIndex>>,
    pub average_metrics: AverageMetrics,
    pub accuracy: Option<f32>,
    pub precision: Option<f32>,
    pub recall: Option<f32>,
    pub timestamp: u64,
}

#[derive(CandidType, CandidDeserialize, Clone, Debug)]
pub struct ClassifierModelData {
    pub data_points: Vec<DataPoint>,
    pub metrics: Metrics,
    pub metrics_history: Vec<Metrics>,
}

#[derive(CandidType, CandidDeserialize, Clone, Debug, PartialEq)]
pub struct LLMModelData {
    pub hugging_face_url: String,
    pub cat_metrics: Option<ContextAssociationTestMetricsBag>,
    pub cat_metrics_history: Vec<ContextAssociationTestMetricsBag>,
    pub evaluations: Vec<ModelEvaluationResult>,
}

#[derive(CandidType, CandidDeserialize, Clone, Debug)]
pub struct CachedThresholds {
    pub thresholds: Option<HashMap<String, (f64, bool)>>
}

#[derive(CandidType, CandidDeserialize, Clone, Debug)]
pub struct Model {
    pub model_id: u128,
    pub model_name: String,
    pub owners: Vec<Principal>,
    pub details: ModelDetails,
    pub model_type: ModelType,
    pub cached_thresholds: Option<CachedThresholds>,
    pub cached_selections: Option<Vec<String>>,
    pub version: u128,
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
    pub description: String,
    pub framework: String,
    pub objective: String,
    pub url: String,
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

#[derive(Serialize, Deserialize, CandidType, Clone, Debug, PartialEq)]
pub struct ContextAssociationTestMetrics {
    pub stereotype: u32,
    pub anti_stereotype: u32,
    pub neutral: u32,
    pub other: u32,
}

#[derive(Serialize, Deserialize, CandidType, Clone, Debug, PartialEq)]
pub struct ContextAssociationTestMetricsBag {
    pub general: ContextAssociationTestMetrics,
    pub intersentence: ContextAssociationTestMetrics,
    pub intrasentence: ContextAssociationTestMetrics,
    pub gender: ContextAssociationTestMetrics,
    pub race: ContextAssociationTestMetrics,
    pub religion: ContextAssociationTestMetrics,
    pub profession: ContextAssociationTestMetrics,
    pub error_count: u32,
    pub error_rate: f32,
    pub total_queries: u32,
    pub timestamp: u64,
    pub intrasentence_prompt_template: String,
    pub intersentence_prompt_template: String,
    pub seed: u32,
    // precalculated fields
    pub icat_score_intra: f32,
    pub icat_score_inter: f32,
    pub icat_score_gender: f32,
    pub icat_score_race: f32,
    pub icat_score_profession: f32,
    pub icat_score_religion: f32,
    pub general_lms: f32,
    pub general_ss: f32,
    pub general_n: u32,
    pub icat_score_general: f32,
    pub data_points: Vec<ContextAssociationTestDataPoint>,
}

#[derive(Serialize, CandidType, CandidDeserialize, Clone, Debug, PartialEq)]
pub enum ContextAssociationTestType {
    Intrasentence,
    Intersentence,
}

#[derive(Serialize, CandidType, CandidDeserialize, Clone, Debug, PartialEq)]
pub struct ContextAssociationTestDataPoint {
    pub data_point_id: u128,
    pub prompt: String,
    pub answer: Option<String>,
    pub result: Option<ContextAssociationTestResult>,
    pub error: bool,
    pub test_type: ContextAssociationTestType,
    pub timestamp: u64,
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

#[derive(Serialize, Debug, CandidType, CandidDeserialize, Clone, PartialEq)]
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

#[derive(Debug, CandidType, CandidDeserialize, Clone)]
pub struct LLMMetricsAPIResult {
    pub metrics: Metrics,
    pub queries: usize,
    pub invalid_responses: u32,
    pub call_errors: u32,
    pub counter_factual: Option<CounterFactualModelEvaluationResult>,
}
