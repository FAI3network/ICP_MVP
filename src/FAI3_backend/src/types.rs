// Use Candid for on-chain data
use candid::{CandidType, Deserialize as CandidDeserialize, Principal};
use ic_stable_structures::storable::Bound;
use ic_stable_structures::Storable;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::collections::HashMap;

pub type PrivilegedMap = HashMap<String, u128>;

#[derive(CandidType, CandidDeserialize, Clone, Debug, PartialEq, Default)]
pub struct JobProgress {
    /// Number of completed iterations of progress.
    pub completed: usize,
    /// Number of target iterations to be reached.
    /// This is usually the same as queries, although some queries can be ignored for
    /// this count, like counter factual queries.
    pub target: usize,
    pub invalid_responses: usize,
    pub call_errors: usize,
}

#[derive(CandidType, CandidDeserialize, Clone, Debug, PartialEq)]
pub struct Job {
    pub id: u128,
    pub model_id: u128,
    pub owner: Principal,
    pub status: String,
    pub timestamp: u64,
    pub job_type: JobType,
    pub status_detail: Option<String>,
    pub progress: JobProgress,
}

impl Storable for Job {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(candid::encode_one(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        match candid::decode_one(&bytes) {
            Ok(job) => job,
            Err(e) => {
                ic_cdk::println!("Error decoding Job: {}", e);
                // Provide a fallback or try alternative decoding approach
                // For now, create a minimal valid job to prevent crashes
                Job {
                    id: 0,
                    model_id: 0,
                    owner: Principal::anonymous(),
                    status: "error_decoding".to_string(),
                    timestamp: 0,
                    job_type: JobType::Unassigned,
                    status_detail: None,
                    progress: JobProgress {
                        completed: 0,
                        target: 0,
                        invalid_responses: 0,
                        call_errors: 0,
                    }
                }
            }
        }
    }

    const BOUND: Bound = Bound::Unbounded;
}

#[derive(CandidType, CandidDeserialize, Clone, Debug, PartialEq)]
pub enum JobType {
    LLMFairness {
        model_evaluation_id: u128,
    },
    ContextAssociationTest {
        metrics_bag_id: u128,
    },
    LanguageEvaluation {
        language_model_evaluation_id: u128,
    },
    AverageFairness {
        job_dependencies: Vec<u128>,
    },
    Unassigned, // used for now for jobs without type
}

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

    pub fn reduce_to_data_points(
        data_points: &Vec<LLMDataPoint>,
        privileged_map: PrivilegedMap,
    ) -> Vec<DataPoint> {
        data_points
            .into_iter()
            .filter_map(|dp| dp.to_data_point(privileged_map.clone()))
            .collect()
    }
}

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq)]
pub struct KeyValuePair {
    pub key: String,
    pub value: u128,
}

impl KeyValuePair {
    pub fn to_hashmap(pairs: Vec<KeyValuePair>) -> std::collections::HashMap<String, u128> {
        pairs.into_iter().map(|pair| (pair.key, pair.value)).collect()
    }
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
    pub queries: usize,
    pub max_queries: usize,
    pub max_errors: u32,
    pub invalid_responses: u32,
    pub errors: u32,
    pub seed: u32,
    pub data_points: Option<Vec<DataPoint>>,
    pub llm_data_points: Option<Vec<LLMDataPoint>>,
    pub prompt_template: Option<String>,
    pub counter_factual: Option<CounterFactualModelEvaluationResult>,
    pub finished: bool,
    pub canceled: bool,
    pub job_id: Option<u128>, 
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
pub struct AverageLLMFairnessMetrics {
    pub model_id: u128,
    pub statistical_parity_difference: f32,
    pub disparate_impact: f32,
    pub average_odds_difference: f32,
    pub equal_opportunity_difference: f32,
    pub accuracy: f32,
    pub precision: f32,
    pub recall: f32,
    pub counter_factual_overall_change_rate: f32,
    // evaluation ids used to calculate this metrics
    pub model_evaluation_ids: Vec<u128>,
}

impl std::fmt::Display for AverageLLMFairnessMetrics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "AverageLLMFairnessMetrics {{ \
             model_id: {}, \
             statistical_parity_difference: {:.4}, \
             disparate_impact: {:.4}, \
             average_odds_difference: {:.4}, \
             equal_opportunity_difference: {:.4}, \
             accuracy: {:.4}, \
             precision: {:.4}, \
             recall: {:.4}, \
             counter_factual_overall_change_rate: {:.4}, \
             model_evaluation_ids: {:?} \
             }}",
            self.model_id,
            self.statistical_parity_difference,
            self.disparate_impact,
            self.average_odds_difference,
            self.equal_opportunity_difference,
            self.accuracy,
            self.precision,
            self.recall,
            self.counter_factual_overall_change_rate,
            self.model_evaluation_ids
        )
    }
}

// TODO: implement display

impl AverageLLMFairnessMetrics {
    // New function initializing with last_computed_evaluation_id as None
    pub fn new(model_id: u128) -> Self {
        AverageLLMFairnessMetrics {
            model_id,
            statistical_parity_difference: 0.0,
            disparate_impact: 0.0,
            average_odds_difference: 0.0,
            equal_opportunity_difference: 0.0,
            accuracy: 0.0,
            precision: 0.0,
            recall: 0.0,
            counter_factual_overall_change_rate: 0.0,
            model_evaluation_ids: Vec::new(),
        }
    }

    /// Adds a new dataset's metrics to the averages, updating each metric and the count.
    pub fn add_metrics(&mut self, model_evaluation: &ModelEvaluationResult) {
        self.model_evaluation_ids
            .push(model_evaluation.model_evaluation_id);

        let metrics = &model_evaluation.metrics;

        // This iterates over the PrivilegedIndex
        // But normally for LLMs, it should have length 1
        if let Some(spd) = &metrics.statistical_parity_difference {
            if !spd.is_empty() {
                self.statistical_parity_difference +=
                    spd.iter().map(|index| index.value).sum::<f32>() / spd.len() as f32;
            }
        }
        if let Some(di) = &metrics.disparate_impact {
            if !di.is_empty() {
                self.disparate_impact +=
                    di.iter().map(|index| index.value).sum::<f32>() / di.len() as f32;
            }
        }
        if let Some(aod) = &metrics.average_odds_difference {
            if !aod.is_empty() {
                self.average_odds_difference +=
                    aod.iter().map(|index| index.value).sum::<f32>() / aod.len() as f32;
            }
        }
        if let Some(eod) = &metrics.equal_opportunity_difference {
            if !eod.is_empty() {
                self.equal_opportunity_difference +=
                    eod.iter().map(|index| index.value).sum::<f32>() / eod.len() as f32;
            }
        }
        if let Some(acc) = metrics.accuracy {
            self.accuracy += acc;
        }
        if let Some(prec) = metrics.precision {
            self.precision += prec;
        }
        if let Some(rec) = metrics.recall {
            self.recall += rec;
        }
        if let Some(counter_factual) = &model_evaluation.counter_factual {
            self.counter_factual_overall_change_rate += counter_factual.change_rate_overall;
        }
    }

    pub fn count(&self) -> usize {
        self.model_evaluation_ids.len()
    }

    /// Finalizes the averages by dividing each summed metric by the count of contributing datasets.
    pub fn finalize_averages(&mut self) {
        let count = self.count() as f32;
        if count > 0.0 {
            self.statistical_parity_difference /= count;
            self.disparate_impact /= count;
            self.average_odds_difference /= count;
            self.equal_opportunity_difference /= count;
            self.accuracy /= count;
            self.precision /= count;
            self.recall /= count;
            self.counter_factual_overall_change_rate /= count;
        }
    }

    /// Returns the last computed evaluation id
    pub fn last_computed_evaluation_id(&self) -> u128 {
        // Returns the max of the model_evaluation_ids linked to this type
        self.model_evaluation_ids.iter().copied().max().unwrap_or(0)
    }

    /// Given a vector of ModelEvaluationResult, it returns the one that was calculated last, ordered by id and filtering by dataset
    /// If no one is found, it returns an error
    pub fn last_computed_evaluation_id_for_dataset(
        evaluations: &Vec<ModelEvaluationResult>,
        dataset: String,
    ) -> Result<ModelEvaluationResult, String> {
        evaluations
            .iter()
            .filter(|e| e.dataset == dataset)
            .max_by(|a, b| a.model_evaluation_id.cmp(&b.model_evaluation_id))
            .cloned()
            .ok_or_else(|| format!("No evaluations found for the dataset `{}`.", dataset))
    }
}

#[derive(CandidType, CandidDeserialize, Clone, Debug, PartialEq)]
pub struct LLMModelData {
    pub hugging_face_url: String,
    pub cat_metrics: Option<ContextAssociationTestMetricsBag>,
    pub cat_metrics_history: Vec<ContextAssociationTestMetricsBag>,
    pub evaluations: Vec<ModelEvaluationResult>,
    pub average_fairness_metrics: Option<AverageLLMFairnessMetrics>,
    pub language_evaluations: Vec<LanguageEvaluationResult>,
    pub inference_provider: Option<String>,
}

impl Default for LLMModelData {
    fn default() -> Self {
        Self {
            hugging_face_url: String::from(""),
            cat_metrics: None,
            cat_metrics_history: Vec::new(),
            evaluations: Vec::new(),
            average_fairness_metrics: None,
            language_evaluations: Vec::new(),
            inference_provider: None,
        }
    }
}

impl LLMModelData {
    /// Returns last model evaluations for some datasets
    pub fn get_last_model_evaluations(
        &self,
        datasets: Vec<String>,
    ) -> Result<Vec<ModelEvaluationResult>, String> {
        if datasets.len() == 0 {
            return Err("Datasets length cannot be zero".to_string());
        }

        let mut ret = Vec::new();

        let mut found = HashMap::<String, ModelEvaluationResult>::new();
        let mut count = 0;

        for evaluation in &self.evaluations {
            if !datasets.contains(&evaluation.dataset) {
                continue;
            }

            if found.contains_key(&evaluation.dataset) {
                if found.get(&evaluation.dataset).unwrap().model_evaluation_id
                    < evaluation.model_evaluation_id
                {
                    found.insert(evaluation.dataset.clone(), evaluation.clone());
                }
            } else {
                found.insert(evaluation.dataset.clone(), evaluation.clone());
                count += 1;
            }
        }

        if count < datasets.len() {
            return Err("Not all required datasets have been found".to_string());
        }

        for key in found.keys() {
            ret.push(found.get(key).unwrap().clone());
        }

        return Ok(ret);
    }
}

#[derive(CandidType, CandidDeserialize, Clone, Debug)]
pub struct CachedThresholds {
    pub thresholds: Option<HashMap<String, (f64, bool)>>,
}

#[derive(CandidType, CandidDeserialize, Clone, Debug)]
pub struct Model {
    pub model_id: u128,
    pub model_name: String,
    pub owners: Vec<Principal>,
    pub details: ModelDetails,
    pub details_history: Vec<ModelDetailsHistory>,
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

impl Model {
    /// Returns a light-weight version of the model, without the heavy data
    /// Like data points, multiple models, etc.
    /// Currently it only prunes LLM models.
    pub fn prune(self) -> Model {
        match self.model_type {
            ModelType::LLM(_) => self.prune_llm_model(),
            _ => self,
        }
    }

    /// Takes a LLM model and returns another model with pruned data
    /// Useful because data_points contain a lot of data
    /// And the protocol doesn't support to return so much data
    pub fn prune_llm_model(mut self) -> Model {
        // Deleting data that could trigger a response size error
        // Error code: IC0504
        let mut model_data = get_llm_model_data(&self);

        model_data.cat_metrics_history = vec![];
        if let Some(mut cat) = model_data.cat_metrics {
            cat.data_points = vec![];
            model_data.cat_metrics = Some(cat);
        }

        model_data.evaluations = model_data
            .evaluations
            .into_iter()
            .map(|mut evaluation: ModelEvaluationResult| {
                evaluation.data_points = None;
                evaluation.llm_data_points = None;
                evaluation
            })
            .collect();

        model_data.language_evaluations = model_data
            .language_evaluations
            .into_iter()
            .map(|mut levaluation: LanguageEvaluationResult| {
                levaluation.data_points = Vec::new();
                levaluation
            })
            .collect();

        self.model_type = ModelType::LLM(model_data);
        self
    }
}

#[derive(CandidType, CandidDeserialize, Clone, Debug)]
pub struct ModelDetails {
    pub description: String,
    pub framework: String,
    pub objective: String,
    pub url: String,
}

#[derive(CandidType, CandidDeserialize, Clone, Debug)]
pub struct UpdatedDetails {
    pub name: String,
    pub details: ModelDetails,
}

#[derive(CandidType, CandidDeserialize, Clone, Debug)]
pub struct ModelDetailsHistory {
    pub(crate) name: String,
    pub(crate) details: ModelDetails,
    pub(crate) version: u128,
    pub(crate) timestamp: u64,
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
    pub context_association_test_id: u128,
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
    pub finished: bool,
    pub canceled: bool,
    pub job_id: Option<u128>,
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

#[derive(CandidType, CandidDeserialize, Clone, Debug, PartialEq)]
pub struct LanguageEvaluationDataPoint {
    pub prompt: String,
    pub response: Option<String>,
    pub valid: bool,
    pub error: bool,
    pub correct_answer: String,
}

#[derive(CandidType, CandidDeserialize, Clone, Debug, PartialEq)]
pub struct LanguageEvaluationResult {
    pub language_model_evaluation_id: u128,
    pub timestamp: u64,
    pub languages: Vec<String>,
    pub data_points: Vec<LanguageEvaluationDataPoint>,
    // Prompt templates might have a different template for every language
    pub prompt_templates: Vec<(String, String)>,
    pub metrics: LanguageEvaluationMetrics,
    pub metrics_per_language: Vec<(String, LanguageEvaluationMetrics)>,
    pub max_queries: usize,
    pub finished: bool,
    pub canceled: bool,
    pub job_id: Option<u128>,
}

#[derive(CandidType, CandidDeserialize, Clone, Debug, PartialEq)]
pub struct LanguageEvaluationMetrics {
    pub overall_accuracy: Option<f32>,
    pub accuracy_on_valid_responses: Option<f32>,
    pub format_error_rate: Option<f32>,
    pub n: u32,
    pub error_count: u32,
    pub invalid_responses: u32,
    pub correct_responses: u32,
    pub incorrect_responses: u32,
}

impl LanguageEvaluationMetrics {
    pub fn new() -> Self {
        Self {
            overall_accuracy: None,
            accuracy_on_valid_responses: None,
            format_error_rate: None,
            n: 0,
            error_count: 0,
            invalid_responses: 0,
            incorrect_responses: 0,
            correct_responses: 0,
        }
    }

    pub fn add_error(&mut self) {
        self.error_count += 1;
    }

    pub fn add_invalid(&mut self) {
        self.invalid_responses += 1;
    }

    pub fn add_incorrect(&mut self) {
        self.incorrect_responses += 1;
    }

    pub fn add_correct(&mut self) {
        self.correct_responses += 1;
    }

    pub fn valid_responses(&self) -> u32 {
        return self.correct_responses + self.incorrect_responses;
    }

    pub fn calculate_rates(&mut self) {
        self.n = self.error_count
            + self.invalid_responses
            + self.incorrect_responses
            + self.correct_responses;

        if self.n > 0 && self.error_count < self.n {
            // overall accuracy does not consider Hugging Face errors.
            self.overall_accuracy = Some(
                self.correct_responses as f32
                    / ((self.correct_responses + self.incorrect_responses + self.invalid_responses)
                        as f32),
            );

            if self.valid_responses() > 0 {
                self.format_error_rate =
                    Some(self.invalid_responses as f32 / (self.n as f32 - self.error_count as f32));
                let valid_responses = self.n - self.invalid_responses - self.error_count;
                if valid_responses > 0 {
                    // Adjust accuracy calculations based on valid responses
                    self.accuracy_on_valid_responses = Some(
                        (self.correct_responses as f32)
                            / ((self.correct_responses + self.incorrect_responses) as f32),
                    );
                } else {
                    self.accuracy_on_valid_responses = Some(0.0);
                }
            }
        }
    }
}
