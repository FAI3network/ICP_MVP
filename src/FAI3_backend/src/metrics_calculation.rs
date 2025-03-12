use crate::types::{PrivilegedIndex, ModelType, get_classifier_model_data};
use crate::{
    check_cycles_before_action, is_owner, DataPoint, MODELS
};

use std::collections::{HashMap, HashSet};

#[ic_cdk::update]
pub(crate) fn calculate_statistical_parity_difference(
    model_id: u128,
    privilieged_threshold: Option<HashMap<String, (f64, bool)>>,
) -> Vec<PrivilegedIndex> {
    check_cycles_before_action();
    let caller = ic_cdk::api::caller();

    MODELS.with(|models| {
        let mut models = models.borrow_mut();
        let mut model = models.get(&model_id).expect("Model not found");
        is_owner(&model, caller);

        let mut model_data = get_classifier_model_data(&model);

        let latest_timestamp = model_data.data_points[model_data.data_points.len() - 1].timestamp;

        let relevant_data_points: Vec<DataPoint> = model_data.data_points
            .iter()
            .filter(|point| point.timestamp == latest_timestamp)
            .cloned()
            .collect();

        let (result, average) = statistical_parity_difference(&relevant_data_points, privilieged_threshold);

        model_data.metrics.average_metrics.statistical_parity_difference = Some(average);

        model_data.metrics.statistical_parity_difference = Some(result.clone());

        // Update timestamp after calculation
        model_data.metrics.timestamp = ic_cdk::api::time();

        model.model_type = ModelType::Classifier(model_data);

        models.insert(model_id, model.clone());

        result
    })
}

#[ic_cdk::update]
pub(crate) fn calculate_disparate_impact(
    model_id: u128,
    privilieged_threshold: Option<HashMap<String, (f64, bool)>>,
) -> Vec<PrivilegedIndex> {
    check_cycles_before_action();
    let caller = ic_cdk::api::caller();

    MODELS.with(|models| {
        let mut models = models.borrow_mut();
        let mut model = models.get(&model_id).expect("Model not found");

        is_owner(&model, caller);

        let mut model_data = get_classifier_model_data(&model);

        let latest_timestamp = model_data.data_points[model_data.data_points.len() - 1].timestamp;

        let relevant_data_points: Vec<DataPoint> = model_data.data_points
            .iter()
            .filter(|point| point.timestamp == latest_timestamp)
            .cloned()
            .collect();

        let (result, average) = disparate_impact(&relevant_data_points, privilieged_threshold);

        model_data.metrics.average_metrics.disparate_impact = Some(average);

        model_data.metrics.disparate_impact = Some(result.clone());

        // Update timestamp after calculation
        model_data.metrics.timestamp = ic_cdk::api::time();

        model.model_type = ModelType::Classifier(model_data);

        models.insert(model_id, model.clone());

        result.clone()
    })
}

#[ic_cdk::update]
pub(crate) fn calculate_average_odds_difference(
    model_id: u128,
    privilieged_threshold: Option<HashMap<String, (f64, bool)>>,
) -> Vec<PrivilegedIndex> {
    check_cycles_before_action();
    let caller = ic_cdk::api::caller();

    return MODELS.with(|models| {
        let mut models = models.borrow_mut();
        let mut model = models.get(&model_id).expect("Model not found");

        is_owner(&model, caller);

        let mut model_data = get_classifier_model_data(&model);

        let latest_timestamp = model_data.data_points[model_data.data_points.len() - 1].timestamp;

        let relevant_data_points: Vec<DataPoint> = model_data.data_points
            .iter()
            .filter(|point| point.timestamp == latest_timestamp)
            .cloned()
            .collect();

        let (result, average) = average_odds_difference(&relevant_data_points, privilieged_threshold);

        model_data.metrics.average_metrics.average_odds_difference = Some(average);

        model_data.metrics.average_odds_difference = Some(result.clone());

        // Update timestamp after calculation
        model_data.metrics.timestamp = ic_cdk::api::time();

        model.model_type = ModelType::Classifier(model_data);

        models.insert(model_id, model.clone());

        result
    });
}

#[ic_cdk::update]
pub(crate) fn calculate_equal_opportunity_difference(
    model_id: u128,
    privilieged_threshold: Option<HashMap<String, (f64, bool)>>,
) -> Vec<PrivilegedIndex> {
    check_cycles_before_action();
    let caller = ic_cdk::api::caller();

    MODELS.with(|models| {
        let mut models = models.borrow_mut();
        let mut model = models.get(&model_id).expect("Model not found");
        is_owner(&model, caller);
        
        let mut model_data = get_classifier_model_data(&model);

        let latest_timestamp = model_data.data_points[model_data.data_points.len() - 1].timestamp;

        let relevant_data_points: Vec<DataPoint> = model_data.data_points
            .iter()
            .filter(|point| point.timestamp == latest_timestamp)
            .cloned()
            .collect();

        let (result, average) = equal_opportunity_difference(&relevant_data_points, privilieged_threshold);

        model_data.metrics.average_metrics.equal_opportunity_difference = Some(average);

        model_data.metrics.equal_opportunity_difference = Some(result.clone());
        model_data.metrics.timestamp = ic_cdk::api::time();
        model.model_type = ModelType::Classifier(model_data);

        models.insert(model_id, model.clone());

        result
    })
}

#[ic_cdk::update]
pub(crate) fn calculate_accuracy(model_id: u128) -> f32 {
    check_cycles_before_action();
    let caller = ic_cdk::api::caller();

    MODELS.with(|models| {
        let mut models = models.borrow_mut();
        let mut model = models.get(&model_id).expect("Model not found");

        is_owner(&model, caller);

        // TODO: work point
        let mut model_data = get_classifier_model_data(&model);

        let latest_timestamp = model_data.data_points[model_data.data_points.len() - 1].timestamp;

        let relevant_data_points: Vec<DataPoint> = model_data.data_points
            .iter()
            .filter(|point| point.timestamp == latest_timestamp)
            .cloned()
            .collect();

        let accuracy = accuracy(&relevant_data_points);
        model_data.metrics.accuracy = Some(accuracy);
        model_data.metrics.timestamp = ic_cdk::api::time();
        model.model_type = ModelType::Classifier(model_data);
        models.insert(model_id, model.clone());

        accuracy
    })
}

#[ic_cdk::update]
pub(crate) fn calculate_precision(model_id: u128) -> f32 {
    check_cycles_before_action();
    let caller = ic_cdk::api::caller();

    MODELS.with(|models| {
        let mut models = models.borrow_mut();
        let mut model = models.get(&model_id).expect("Model not found");

        is_owner(&model, caller);
        
        let mut model_data = get_classifier_model_data(&model);
        let latest_timestamp = model_data.data_points[model_data.data_points.len() - 1].timestamp;

        let relevant_data_points: Vec<DataPoint> = model_data.data_points
            .iter()
            .filter(|point| point.timestamp == latest_timestamp)
            .cloned()
            .collect();

        let precision = precision(&relevant_data_points);
        model_data.metrics.precision = Some(precision);
        model_data.metrics.timestamp = ic_cdk::api::time();
        
        model.model_type = ModelType::Classifier(model_data);
        models.insert(model_id, model.clone());

        precision
    })
}

#[ic_cdk::update]
pub(crate) fn calculate_recall(model_id: u128) -> f32 {
    check_cycles_before_action();
    let caller = ic_cdk::api::caller();

    MODELS.with(|models| {
        let mut models = models.borrow_mut();
        let mut model = models.get(&model_id).expect("Model not found");

        is_owner(&model, caller);
        let mut model_data = get_classifier_model_data(&model);
        let latest_timestamp = model_data.data_points[model_data.data_points.len() - 1].timestamp;

        let relevant_data_points: Vec<DataPoint> = model_data.data_points
            .iter()
            .filter(|point| point.timestamp == latest_timestamp)
            .cloned()
            .collect();

        let recall = recall(&relevant_data_points);
        model_data.metrics.recall = Some(recall);
        model_data.metrics.timestamp = ic_cdk::api::time();
        model.model_type = ModelType::Classifier(model_data);
        models.insert(model_id, model.clone());

        // Push the updated metrics to the history
        recall
    })
}

#[ic_cdk::update]
/// Calculates all relevant metrics for a given model.
/// 
/// This function computes a series of fairness and performance metrics for a specific model based on
/// the model_id and an optional threshold provided for privileged groups.
/// It returns the Statistical Parity Difference, Disparate Impact, Average Odds Difference,
/// Equal Opportunity Difference, Accuracy, Precision, and Recall.
/// 
/// - model_id: The unique identifier for the model.
/// - privilieged_threshold: An optional HashMap where keys are feature names with their threshold values and a boolean
///   indicating if higher values are privileged.
/// 
/// Returns a tuple of several metrics results in the following order:
/// 1. Vector of Statistical Parity Difference for each group
/// 2. Vector of Disparate Impact for each group
/// 3. Vector of Average Odds Difference for each group
/// 4. Vector of Equal Opportunity Difference for each group
/// 5. Accuracy as a single f32 value
/// 6. Precision as a single f32 value
/// 7. Recall as a single f32 value
pub(crate) fn calculate_all_metrics(
    model_id: u128,
    privilieged_threshold: Option<HashMap<String, (f64, bool)>>,
) -> (
    Vec<PrivilegedIndex>,
    Vec<PrivilegedIndex>,
    Vec<PrivilegedIndex>,
    Vec<PrivilegedIndex>,
    f32,
    f32,
    f32,
) {
    let spd = calculate_statistical_parity_difference(model_id, privilieged_threshold.clone());
    let di = calculate_disparate_impact(model_id, privilieged_threshold.clone());
    let aod = calculate_average_odds_difference(model_id, privilieged_threshold.clone());
    let eod = calculate_equal_opportunity_difference(model_id, privilieged_threshold);
    let acc = calculate_accuracy(model_id);
    let prec = calculate_precision(model_id);
    let rec = calculate_recall(model_id);

    MODELS.with(|models| {
        let mut models = models.borrow_mut();
        let mut model = models.get(&model_id).expect("Model not found");
        let mut model_data = get_classifier_model_data(&model);
        model_data.metrics.timestamp = ic_cdk::api::time();
        model_data.metrics_history.push(model_data.metrics.clone());
        model.model_type = ModelType::Classifier(model_data);
        models.insert(model_id, model.clone());
    });

    (spd, di, aod, eod, acc, prec, rec)
}

/// Calculates group counts for privileged and unprivileged groups based on specified thresholds.
///
/// This function separates the data points into privileged and unprivileged groups according to 
/// the thresholds provided or calculated median values. It then counts the total number and the 
/// number of positive outcomes for both groups.
///
/// # Arguments
/// * `data_points` - A reference to a vector of `DataPoint` containing features and outcomes.
/// * `privileged_threshold` - Optional hash map defining the thresholds for determining whether 
///   a data point is privileged or not.
///
/// # Returns
/// Four hash maps containing counts of:
/// - Total privileged
/// - Total unprivileged
/// - Privileged positive outcomes
/// - Unprivileged positive outcomes
pub(crate) fn calculate_group_counts(
    data_points: &Vec<DataPoint>,
    privilieged_threshold: Option<HashMap<String, (f64, bool)>>,
) -> (
    HashMap<String, u128>,
    HashMap<String, u128>,
    HashMap<String, u128>,
    HashMap<String, u128>,
) {
    let mut privileged_count_list: HashMap<String, u128> = HashMap::new();
    let mut unprivileged_count_list: HashMap<String, u128> = HashMap::new();
    let mut privileged_positive_count_list: HashMap<String, u128> = HashMap::new();
    let mut unprivileged_positive_count_list: HashMap<String, u128> = HashMap::new();

    let threshold_map = if privilieged_threshold.is_some() {
        privilieged_threshold.unwrap()
    } else {
        calculate_medians(data_points)
    };

    for point in data_points {
        let features_list = point.features.clone();

        for entry in &point.privileged_map {
            let vairable_name = entry.0;
            let variable_index = entry.1;

            let threshold = *threshold_map.get(vairable_name).unwrap_or(&(0.0, true));

            let greater_than = threshold.1;

            let value = features_list[*variable_index as usize];
            let is_privileged = if greater_than {
                value > threshold.0
            } else {
                value < threshold.0
            };

            if is_privileged {
                privileged_count_list
                    .entry(vairable_name.clone())
                    .and_modify(|e| *e += 1)
                    .or_insert(1);
                if point.predicted {
                    privileged_positive_count_list
                        .entry(vairable_name.clone())
                        .and_modify(|e| *e += 1)
                        .or_insert(1);
                }
            } else {
                unprivileged_count_list
                    .entry(vairable_name.clone())
                    .and_modify(|e| *e += 1)
                    .or_insert(1);
                if point.predicted {
                    unprivileged_positive_count_list
                        .entry(vairable_name.clone())
                        .and_modify(|e| *e += 1)
                        .or_insert(1);
                }
            }
        }
    }

    (
        privileged_count_list,
        unprivileged_count_list,
        privileged_positive_count_list,
        unprivileged_positive_count_list,
    )
}

/// Calculates the confusion matrix for provided data points based on optional privileged thresholds.
///
/// # Arguments
/// * `data_points` - A reference to a vector of `DataPoint` structures.
/// * `privileged_threshold` - An optional map from keys to a pair of floating points and booleans, indicating thresholds and conditions.
pub(crate) fn calculate_confusion_matrix(
    data_points: &Vec<DataPoint>,
    privilieged_threshold: Option<HashMap<String, (f64, bool)>>,
) -> (
    HashMap<String, u128>,
    HashMap<String, u128>,
    HashMap<String, u128>,
    HashMap<String, u128>,
    HashMap<String, u128>,
    HashMap<String, u128>,
    HashMap<String, u128>,
    HashMap<String, u128>,
) {
    let (mut privileged_tp, mut privileged_fp, mut privileged_tn, mut privileged_fn) = (
        HashMap::new(),
        HashMap::new(),
        HashMap::new(),
        HashMap::new(),
    );
    let (mut unprivileged_tp, mut unprivileged_fp, mut unprivileged_tn, mut unprivileged_fn) = (
        HashMap::new(),
        HashMap::new(),
        HashMap::new(),
        HashMap::new(),
    );

    let threshold_map = if privilieged_threshold.is_some() {
        privilieged_threshold.unwrap()
    } else {
        calculate_medians(data_points)
    };

    for point in data_points {
        let features_list = point.features.clone();
        for entry in point.privileged_map.iter() {
            let vairable_name = entry.0;
            let variable_index = entry.1;

            let threshold = *threshold_map.get(vairable_name).unwrap_or(&(0.0, true));

            let greater_than = threshold.1;

            let value = features_list[*variable_index as usize];

            let is_privileged = if greater_than {
                value > threshold.0
            } else {
                value < threshold.0
            };

            match (point.target, point.predicted) {
                (true, true) => {
                    if is_privileged {
                        privileged_tp
                            .entry(vairable_name.clone())
                            .and_modify(|e| *e += 1)
                            .or_insert(1);
                    } else {
                        unprivileged_tp
                            .entry(vairable_name.clone())
                            .and_modify(|e| *e += 1)
                            .or_insert(1);
                    }
                }
                (true, false) => {
                    if is_privileged {
                        privileged_fn
                            .entry(vairable_name.clone())
                            .and_modify(|e| *e += 1)
                            .or_insert(1);
                    } else {
                        unprivileged_fn
                            .entry(vairable_name.clone())
                            .and_modify(|e| *e += 1)
                            .or_insert(1);
                    }
                }
                (false, true) => {
                    if is_privileged {
                        privileged_fp
                            .entry(vairable_name.clone())
                            .and_modify(|e| *e += 1)
                            .or_insert(1);
                    } else {
                        unprivileged_fp
                            .entry(vairable_name.clone())
                            .and_modify(|e| *e += 1)
                            .or_insert(1);
                    }
                }
                (false, false) => {
                    if is_privileged {
                        privileged_tn
                            .entry(vairable_name.clone())
                            .and_modify(|e| *e += 1)
                            .or_insert(1);
                    } else {
                        unprivileged_tn
                            .entry(vairable_name.clone())
                            .and_modify(|e| *e += 1)
                            .or_insert(1);
                    }
                }
            }
        }
    }

    (
        privileged_tp,
        privileged_fp,
        privileged_tn,
        privileged_fn,
        unprivileged_tp,
        unprivileged_fp,
        unprivileged_tn,
        unprivileged_fn,
    )
}

/// Calculates the overall confusion matrix from a set of data points.
///
/// # Parameters
/// * `data_points` - A reference to a vector of `DataPoint` structs containing the target and predicted values.
///
/// # Returns
/// Returns a tuple `(tp, tn, fp, fn_)` representing true positives, true negatives, false positives, and false negatives respectively.
pub(crate) fn calculate_overall_confusion_matrix(
    data_points: &Vec<DataPoint>,
) -> (i128, i128, i128, i128) {
    let (mut tp, mut tn, mut fp, mut fn_) = (0, 0, 0, 0);

    for point in data_points {
        match (point.target, point.predicted) {
            (true, true) => tp += 1,
            (false, false) => tn += 1,
            (false, true) => fp += 1,
            (true, false) => fn_ += 1,
        }
    }

    (tp, tn, fp, fn_)
}


/// Calculates the median values for each variable in a dataset, and marks them as valid.
///
/// This function processes a dataset provided as a vector of `DataPoint` entries
/// and computes the median value for each variable inside these entries.
/// All variable names and their associated median values are stored in a
/// `HashMap` where the key is the variable's name and the value is a tuple
/// containing the median value and a boolean.
///
/// # Parameters
/// - `data_points`: A reference to a vector of `DataPoint` structures which contains
///   the feature array and a map of variable indices identifying their locations in the features array.
///
/// # Returns
/// A `HashMap` where each key is a string representing the variable name,
/// and each value is a tuple containing the median value and a boolean indicating if it is valid.
pub(crate) fn calculate_medians(data_points: &Vec<DataPoint>) -> HashMap<String, (f64, bool)> {
    let mut medians: HashMap<String, (f64, bool)> = HashMap::new();
    let mut variable_values: HashMap<String, Vec<f64>> = HashMap::new();

    for point in data_points {
        for entry in point.privileged_map.iter() {
            let variable_name = entry.0;
            let variable_index = entry.1;

            let value = point.features[*variable_index as usize];

            if value.is_nan() {
                continue;
            }

            variable_values
                .entry(variable_name.clone())
                .and_modify(|e| e.push(value))
                .or_insert(vec![value]);
        }
    }

    for (key, value) in variable_values.iter_mut() {
        value.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let min = value.first().cloned().unwrap_or(0.0);
        let max = value.last().cloned().unwrap_or(0.0);
        let middle_of_range = (min + max) / 2.0;
        medians
            .entry(key.clone())
            .or_insert((middle_of_range, true));
    }

    medians
}


/// Calculates the Statistical Parity Difference (SPD) across different groups within the provided data.
///
/// # Parameters:
/// * `data_points`: A reference to a vector of `DataPoint` instances, representing the dataset.
/// * `privileged_threshold`: An optional HashMap defining the privileged threshold for different variable names.
///   Each entry in the map specifies a threshold value and a boolean indicating whether higher values than the threshold
///   are considered privileged.
///
/// # Returns:
/// A tuple containing:
/// * A vector of `PrivilegedIndex`, where each `PrivilegedIndex` holds the variable name and the calculated SPD.
/// * A floating-point number (f32) representing the average SPD across all variables.
pub(crate) fn statistical_parity_difference(data_points: &Vec<DataPoint>, privilieged_threshold: Option<HashMap<String, (f64, bool)>>) -> (Vec<PrivilegedIndex>, f32) {
    let (
        privileged_count,
        unprivileged_count,
        privileged_positive_count,
        unprivileged_positive_count,
    ) = calculate_group_counts(&data_points, privilieged_threshold);

    // Handle empty group scenario
    if privileged_count.len() == 0 || unprivileged_count.len() == 0 {
        ic_cdk::api::trap(
            "Cannot calculate statistical parity difference: One of the groups has no data points.",
        );
    }

    let mut result = Vec::new();

    let all_keys: HashSet<&String> = privileged_count
        .keys()
        .chain(unprivileged_count.keys())
        .collect();

    for key in all_keys {
        let privileged_total = *privileged_count.get(key).unwrap_or(&0) as f32;
        let unprivileged_total = *unprivileged_count.get(key).unwrap_or(&0) as f32;

        let privileged_positives = *privileged_positive_count.get(key).unwrap_or(&0) as f32;
        let unprivileged_positives = *unprivileged_positive_count.get(key).unwrap_or(&0) as f32;

        // Avoid division by zero
        if privileged_total == 0.0 || unprivileged_total == 0.0 {
            // result.insert(key.clone(), None); // Skip SPD for this variable
            continue;
        }

        let privileged_probability = privileged_positives / privileged_total;
        let unprivileged_probability = unprivileged_positives / unprivileged_total;

        let diff = unprivileged_probability - privileged_probability;

        let new_entry = PrivilegedIndex {
            variable_name: key.clone(),
            value: diff,
        };

        result.push(new_entry);
    }

    let sum: f32 = result.iter().map(|x| x.value).sum();
    let length: f32 = result.len() as f32;

    let average: f32 = sum / length;

    (result, average)
}

/// Calculates the Disparate Impact (DI) measure between different groups within the data.
///
/// # Arguments
///
/// * `data_points` - A reference to a vector of `DataPoint` instances, which represent the data points to evaluate.
/// * `privileged_threshold` - An optional hashmap where each key represents a variable name and each value is a tuple composed of a threshold value and a boolean indicating if the privilege comparisson should use greater or lower.
///
/// # Returns
///
/// A tuple containing:
/// - A vector of `PrivilegedIndex` structures, where each holds a variable name and its DI score.
/// - A single float (f32) representing the average DI score across all variables.
///
pub(crate) fn disparate_impact(data_points: &Vec<DataPoint>, privilieged_threshold: Option<HashMap<String, (f64, bool)>>) -> (Vec<PrivilegedIndex>, f32) {
    let (
        privileged_count,
        unprivileged_count,
        privileged_positive_count,
        unprivileged_positive_count,
    ) = calculate_group_counts(&data_points, privilieged_threshold);

    if privileged_count.len() == 0 || unprivileged_count.len() == 0 {
        ic_cdk::api::trap(
            "Cannot calculate statistical parity difference: One of the groups has no data points.",
        );
    }

    let mut result = Vec::new();

    let all_keys: HashSet<&String> = privileged_count
        .keys()
        .chain(unprivileged_count.keys())
        .collect();

    for key in all_keys {
        let privileged_total = *privileged_count.get(key).unwrap_or(&0) as f32;
        let unprivileged_total = *unprivileged_count.get(key).unwrap_or(&0) as f32;

        let privileged_positives = *privileged_positive_count.get(key).unwrap_or(&0) as f32;
        let unprivileged_positives = *unprivileged_positive_count.get(key).unwrap_or(&0) as f32;

        // Avoid division by zero
        if privileged_total == 0.0 || unprivileged_total == 0.0 {
            // result.insert(key.clone(), None); // Skip SPD for this variable
            continue;
        }

        let privileged_probability = privileged_positives / privileged_total;
        let unprivileged_probability = unprivileged_positives / unprivileged_total;

        let diff = unprivileged_probability / privileged_probability;

        let new_entry = PrivilegedIndex {
            variable_name: key.clone(),
            value: diff,
        };

        result.push(new_entry);
    }

    let sum: f32 = result.iter().map(|x| x.value).sum();
    let length: f32 = result.len() as f32;

    let average: f32 = sum / length;

    (result, average)
}

/// Computes the average odds difference for a given dataset considering potentially
/// privileged groups thresholded based on the provided criteria.
///
/// # Arguments
/// * `data_points` - A reference to a vector containing the data points to evaluate.
/// * `privileged_threshold` - An optional hashmap where each key corresponds to a group identifier
///   and the value is a tuple containing a numeric threshold and a boolean indicating the privilege direction.
///
/// # Returns
/// A tuple containing a vector of indices for privileged data points and a floating-point
/// representation of the average odds difference.
pub(crate) fn average_odds_difference(data_points: &Vec<DataPoint>, privilieged_threshold: Option<HashMap<String, (f64, bool)>>) -> (Vec<PrivilegedIndex>, f32) { 
    let (
        privileged_tp,
        privileged_fp,
        privileged_tn,
        privileged_fn,
        unprivileged_tp,
        unprivileged_fp,
        unprivileged_tn,
        unprivileged_fn,
    ) = calculate_confusion_matrix(&data_points, privilieged_threshold);

    let mut result = Vec::new();

    for (key, _) in &privileged_tp {
        let privileged_positive_total =
            *privileged_tp.get(key).unwrap_or(&0) + *privileged_fn.get(key).unwrap_or(&0);
        let unprivileged_positive_total =
            *unprivileged_tp.get(key).unwrap_or(&0) + *unprivileged_fn.get(key).unwrap_or(&0);
        let privileged_negative_total =
            *privileged_fp.get(key).unwrap_or(&0) + *privileged_tn.get(key).unwrap_or(&0);
        let unprivileged_negative_total =
            *unprivileged_fp.get(key).unwrap_or(&0) + *unprivileged_tn.get(key).unwrap_or(&0);

        // if privileged_positive_total == 0 || unprivileged_positive_total == 0 || privileged_negative_total == 0 || unprivileged_negative_total == 0 {
        //     ic_cdk::api::trap("Cannot calculate average odds difference: One of the groups has no data points or no positives/negatives.");
        // }

        let privileged_tpr: f32 = *privileged_tp.get(key).unwrap_or(&0) as f32
            / (privileged_positive_total + 1) as f32;
        let unprivileged_tpr: f32 = *unprivileged_tp.get(key).unwrap_or(&0) as f32
            / (unprivileged_positive_total + 1) as f32;
        let privileged_fpr: f32 = *privileged_fp.get(key).unwrap_or(&0) as f32
            / (privileged_negative_total + 1) as f32;
        let unprivileged_fpr: f32 = *unprivileged_fp.get(key).unwrap_or(&0) as f32
            / (unprivileged_negative_total + 1) as f32;

        let diff = ((unprivileged_fpr - privileged_fpr).abs()
                    + (unprivileged_tpr - privileged_tpr).abs())
            / 2.0;

        let new_entry = PrivilegedIndex {
            variable_name: key.clone(),
            value: diff,
        };

        result.push(new_entry);
    }

    let sum: f32 = result.iter().map(|x| x.value).sum();
    let length: f32 = result.len() as f32;

    if result.len() == 0 {
        ic_cdk::trap("No data to calculate average odds difference.");
    }

    let average: f32 = sum / length;

    (result, average)
}
                               
/// Computes the difference in opportunities between privileged and unprivileged groups.
///
/// # Arguments
/// * `data_points` - A reference to a vector of `DataPoint` structs that contain the relevant data.
/// * `privilieged_threshold` - An optional hash map where the key is a characteristic and the value is a tuple containing a threshold and a flag determining the privilege direction.
///
/// # Returns
/// * A tuple containing:
///   - `Vec<PrivilegedIndex>`: A vector indicating the indices of privileged data points.
///   - `f32`: A floating-point number representing the difference in opportunities.
pub(crate) fn equal_opportunity_difference(data_points: &Vec<DataPoint>, privilieged_threshold: Option<HashMap<String, (f64, bool)>>) -> (Vec<PrivilegedIndex>, f32) {
    let mut count_pred_label_unprivileged = HashMap::new();
    let mut count_pred_label_privileged = HashMap::new();
    let mut count_label_unprivileged = HashMap::new();
    let mut count_label_privileged = HashMap::new();

    let threshold_map = if privilieged_threshold.is_some() { privilieged_threshold.unwrap() } else { calculate_medians(&data_points) };

    
    for point in data_points {
        for entry in point.privileged_map.iter() {
            let vairable_name = entry.0;
            let variable_index = entry.1;

            let threshold = *threshold_map.get(vairable_name).unwrap_or(&(0.0, true));

            let greater_than = threshold.1;

            let value = point.features[*variable_index as usize];
            
            let is_privileged = if greater_than {
                value > threshold.0
            } else {
                value < threshold.0
            };

            if is_privileged {
                if point.target {
                    count_label_privileged
                        .entry(vairable_name.clone())
                        .and_modify(|e| *e += 1.0)
                        .or_insert(1.0);
                    if point.predicted {
                        count_pred_label_privileged
                            .entry(vairable_name.clone())
                            .and_modify(|e| *e += 1.0)
                            .or_insert(1.0);
                    }
                }
            } else {
                if point.target {
                    count_label_unprivileged
                        .entry(vairable_name.clone())
                        .and_modify(|e| *e += 1.0)
                        .or_insert(1.0);
                    if point.predicted {
                        count_pred_label_unprivileged
                            .entry(vairable_name.clone())
                            .and_modify(|e| *e += 1.0)
                            .or_insert(1.0);
                    }
                }
            }
        }
    }

    let mut result = Vec::new();

    let all_keys: HashSet<&String> = count_label_privileged
        .keys()
        .chain(count_label_unprivileged.keys())
        .collect();

    for key in all_keys {
        let prob_pred_label_unprivileged =
            *count_pred_label_unprivileged.get(key).unwrap_or(&0.0)
            / (*count_label_unprivileged.get(key).unwrap_or(&0.0)); // TODO: +1 removed
        let prob_pred_label_privileged = *count_pred_label_privileged.get(key).unwrap_or(&0.0)
            / (*count_label_privileged.get(key).unwrap_or(&0.0)); // TODO: +1 removed

        let diff = prob_pred_label_unprivileged - prob_pred_label_privileged;

        let new_entry = PrivilegedIndex {
            variable_name: key.clone(),
            value: diff,
        };

        result.push(new_entry);
    }

    let sum: f32 = result.iter().map(|x| x.value).sum();
    let length: f32 = result.len() as f32;

    let average: f32 = sum / length;

    (result, average)
}

pub(crate) fn accuracy(data_points: &Vec<DataPoint>) -> f32 {
    let (tp, tn, fp, fn_) = calculate_overall_confusion_matrix(&data_points);
    let total = tp + tn + fp + fn_;
    if total == 0 {
        ic_cdk::api::trap("No data points to calculate accuracy");
    }

    return (tp + tn) as f32 / total as f32;
}

pub(crate) fn can_calculate_precision(tp: i128, fp: i128) -> bool {
    let denominator = tp + fp;
    return denominator != 0;
}

pub(crate) fn can_calculate_recall(tp: i128, fn_: i128) -> bool {
    let denominator = tp + fn_;
    return denominator != 0;
}


pub(crate) fn precision(data_points: &Vec<DataPoint>) -> f32 {
    let (tp, _, fp, _) = calculate_overall_confusion_matrix(&data_points);
    let denominator = tp + fp;
    if !can_calculate_precision(tp, fp) {
        ic_cdk::api::trap("No positive predictions for precision");
    }

    return tp as f32 / denominator as f32;
}

pub(crate) fn recall(data_points: &Vec<DataPoint>) -> f32 {
    let (tp, _, _, fn_) = calculate_overall_confusion_matrix(&data_points);
    let denominator = tp + fn_;
    if !can_calculate_recall(tp, fn_) {
        ic_cdk::api::trap("No actual positives for recall");
    }

    return tp as f32 / denominator as f32;
}

pub(crate) fn all_metrics(data_points: &Vec<DataPoint>, privilieged_threshold: Option<HashMap<String, (f64, bool)>>) -> ((Vec<PrivilegedIndex>, f32), (Vec<PrivilegedIndex>, f32), (Vec<PrivilegedIndex>, f32), (Vec<PrivilegedIndex>, f32), f32, f32, f32) {
    let spd = statistical_parity_difference(&data_points, privilieged_threshold.clone());
    let di = disparate_impact(&data_points, privilieged_threshold.clone());
    let aod = average_odds_difference(&data_points, privilieged_threshold.clone());
    let eod = equal_opportunity_difference(&data_points, privilieged_threshold);
    let acc = accuracy(&data_points);
    let prec = precision(&data_points);
    let rec = recall(&data_points);

    (spd, di, aod, eod, acc, prec, rec)
}

#[cfg(test)]
mod metrics_calculation_tests {
    use std::collections::HashMap;
    use super::*;

    fn gender_pmap() -> HashMap<String, u128> {
        return HashMap::from([(String::from("gender"), 0)]);
    }
    
    
    fn mock_data_points_all_correct() -> Vec<DataPoint> {
        
        // Perfect classifier: target == predicted
        vec![
            DataPoint {
                data_point_id: 1,
                target: true,
                predicted: true,
                privileged_map: gender_pmap(),
                features: vec![0.5],
                timestamp: 0,
            },
            DataPoint {
                data_point_id: 2,
                target: false,
                predicted: false,
                privileged_map: gender_pmap(),
                features: vec![1.0],
                timestamp: 0,
            },
        ]
    }

    fn mock_data_points_stat_parity_example() -> Vec<DataPoint> {
        // Privileged group: predicted positives in half
        // Unprivileged group: predicted positives in all
        vec![
            DataPoint {
                data_point_id: 1,
                target: true,
                predicted: true,
                privileged_map: gender_pmap(),
                features: vec![1.0],
                timestamp: 0,
            },
            DataPoint {
                data_point_id: 2,
                target: false,
                predicted: false,
                privileged_map: gender_pmap(),
                features: vec![1.0],
                timestamp: 0,
            },
            DataPoint {
                data_point_id: 3,
                target: true,
                predicted: true,
                privileged_map: gender_pmap(),
                features: vec![0.0],
                timestamp: 0,
            },
            DataPoint {
                data_point_id: 4,
                target: false,
                predicted: true,
                privileged_map: gender_pmap(),
                features: vec![0.0],
                timestamp: 0,
            },
        ]
    }

    fn mock_data_points_disparate_impact() -> Vec<DataPoint> {
        // Privileged: 2 data points, 1 predicted positive => P(Pos|Priv) = 0.5
        // Unprivileged: 2 data points, 2 predicted positive => P(Pos|Unpriv) = 1.0
        // DI = 1.0 / 0.5 = 2.0
        mock_data_points_stat_parity_example()
    }

    fn mock_data_points_average_odds() -> Vec<DataPoint> {
        // Privileged: TP=1, FN=1, FP=1, TN=1 => TPR=1/2=0.5, FPR=1/2=0.5
        // Unprivileged: identical to privileged
        vec![
            // Privileged:
            DataPoint { data_point_id: 1, target: true, predicted: true, privileged_map: gender_pmap(), features: vec![1.0], timestamp: 0 }, // TP
            DataPoint { data_point_id: 2, target: true, predicted: false, privileged_map: gender_pmap(), features: vec![1.0], timestamp: 0 }, // FN
            DataPoint { data_point_id: 3, target: false, predicted: true, privileged_map: gender_pmap(), features: vec![1.0], timestamp: 0 }, // FP
            DataPoint { data_point_id: 4, target: false, predicted: false, privileged_map: gender_pmap(), features: vec![1.0], timestamp: 0 }, // TN

            // Unprivileged (exact same pattern):
            DataPoint { data_point_id: 5, target: true, predicted: true, privileged_map: gender_pmap(), features: vec![0.0], timestamp: 0 }, // TP
            DataPoint { data_point_id: 6, target: true, predicted: false, privileged_map: gender_pmap(), features: vec![0.0], timestamp: 0 }, // FN
            DataPoint { data_point_id: 7, target: false, predicted: true, privileged_map: gender_pmap(), features: vec![0.0], timestamp: 0 }, // FP
            DataPoint { data_point_id: 8, target: false, predicted: false, privileged_map: gender_pmap(), features: vec![0.0], timestamp: 0 }, // TN
        ]
    }

    fn mock_data_points_eod() -> Vec<DataPoint> {
        // Equal Opportunity Difference test:
        // If privileged TPR=1.0, unprivileged TPR=0.5, EOD=0.5 - 1.0 = -0.5
        vec![
            // Privileged all true positives (TP=2, FN=0)
            DataPoint { data_point_id: 1, target: true, predicted: true, privileged_map: gender_pmap(), features: vec![1.0], timestamp: 0 },
            DataPoint { data_point_id: 2, target: true, predicted: true, privileged_map: gender_pmap(), features: vec![1.0], timestamp: 0 },

            // Unprivileged (TP=1, FN=1)
            DataPoint { data_point_id: 3, target: true, predicted: true, privileged_map: gender_pmap(), features: vec![0.0], timestamp: 0 },
            DataPoint { data_point_id: 4, target: true, predicted: false, privileged_map: gender_pmap(), features: vec![0.0], timestamp: 0 },
        ]
    }

    fn mock_data_points_precision() -> Vec<DataPoint> {
        // Precision = TP / (TP+FP)
        // Let's say TP=2, FP=2 => Precision = 0.5
        vec![
            DataPoint { data_point_id: 1, target: true, predicted: true, privileged_map: gender_pmap(), features: vec![0.0], timestamp: 0 },
            DataPoint { data_point_id: 2, target: true, predicted: true, privileged_map: gender_pmap(), features: vec![0.0], timestamp: 0 },
            DataPoint { data_point_id: 3, target: false, predicted: true, privileged_map: gender_pmap(), features: vec![0.0], timestamp: 0 },
            DataPoint { data_point_id: 4, target: false, predicted: true, privileged_map: gender_pmap(), features: vec![0.0], timestamp: 0 },
        ]
    }

    fn mock_data_points_recall() -> Vec<DataPoint> {
        // Recall = TP / (TP+FN)
        // Let's say TP=2, FN=2 => Recall = 0.5
        vec![
            DataPoint { data_point_id: 1, target: true, predicted: true, privileged_map: gender_pmap(), features: vec![0.0], timestamp: 0 },
            DataPoint { data_point_id: 2, target: true, predicted: true, privileged_map: gender_pmap(), features: vec![0.0], timestamp: 0 },
            DataPoint { data_point_id: 3, target: true, predicted: false, privileged_map: gender_pmap(), features: vec![0.0], timestamp: 0 },
            DataPoint { data_point_id: 4, target: true, predicted: false, privileged_map: gender_pmap(), features: vec![0.0], timestamp: 0 },
        ]
    }


    #[cfg(test)]
    mod test_accuracy {
        use super::*;
        // Assuming this function exists

        #[test]
        fn test_perfect_accuracy() {
            let data_points = mock_data_points_all_correct();
            // Assuming calculate_accuracy takes model_id and updates model.metrics.accuracy
            // If it's a pure function, adjust accordingly.
            let acc = accuracy(&data_points);
            assert!((acc - 1.0).abs() < 1e-6, "Accuracy should be 1.0 for a perfect classifier");
        }

        #[test]
        #[should_panic(expected = "trap should only be called inside canisters.")]
        fn test_no_data_points() {
            let data_points: Vec<DataPoint> = vec![];
            let _acc = accuracy(&data_points); // Should panic
        }
    }

    #[cfg(test)]
    mod test_statistical_parity {
        use super::*;

        #[test]
        fn test_statistical_parity_difference_basic() {
            let data_points = mock_data_points_stat_parity_example();

            // Suppose in this example:
            // Privileged group: 2 data points, 1 predicted positive => P(Pos|Priv) = 0.5
            // Unprivileged group: 2 data points, 2 predicted positive => P(Pos|Unpriv) = 1.0
            // SPD = 1.0 - 0.5 = 0.5
            let (_, spd) = statistical_parity_difference(&data_points, None);
            println!("{}",spd);
            assert!((spd - 0.5).abs() < 1e-6, "Statistical Parity Difference should be 0.5");
        }

        #[test]
        #[should_panic(expected = "trap should only be called inside canisters.")]
        fn test_all_privileged_no_unprivileged() {
            let data_points = vec![
                DataPoint {
                    data_point_id: 1,
                    target: true,
                    predicted: true,
                    privileged_map: gender_pmap(),
                    features: vec![1.0],
                    timestamp: 0,
                }
            ];
            let _spd = statistical_parity_difference(&data_points, None); // Should panic
        }
    }

    #[cfg(test)]
    mod test_disparate_impact {
        use super::*;
        
        #[test]
        fn test_disparate_impact_basic() {
            let data = mock_data_points_disparate_impact();
            // DI expected = (1.0 / 0.5) = 2.0
            // DI expected = (1.0 / 0.5+1) = 0.66
            let (_, di) = disparate_impact(&data, None);
            println!("{}",di);
            assert!((di - 2.0).abs() < 1e-6, "Disparate Impact should be 2.0");
        }

        #[test]
        #[should_panic(expected = "trap should only be called inside canisters.")]
        fn test_no_group_data_di() {
            let data = vec![]; // no data at all
            let _di = disparate_impact(&data, None);
        }
    }

    #[cfg(test)]
    mod test_average_odds_difference {
        use super::*;

        #[test]
        fn test_average_odds_difference_basic() {
            let data = mock_data_points_average_odds(); // updated version
            let (_, aod) = average_odds_difference(&data, None);
            assert!((aod - 0.0).abs() < 1e-6, "Average Odds Difference should be 0.0");
        }

        #[test]
        #[should_panic(expected = "trap should only be called inside canisters.")]
        fn test_aod_missing_data() {
            let data: Vec<DataPoint> = vec![];
            let _aod = average_odds_difference(&data, None);
        }
    }

    #[cfg(test)]
    mod test_equal_opportunity_difference {
        use super::*;

        #[test]
        fn test_eod_basic() {
            let data = mock_data_points_eod();
            // Privileged TPR = 2/2 =1.0
            // Unpriv TPR = 1/2=0.5
            // EOD = 0.5 -1.0 = -0.5
            let (_, eod) = equal_opportunity_difference(&data, None);
            println!("{}", eod);
            assert!((eod + 0.5).abs() < 1e-6, "Equal Opportunity Difference should be -0.5");
        }

        // #[test]
        // #[should_panic(expected = "One of the groups has no positive data points")]
        // fn test_eod_no_positives() {
        //     let data: Vec<DataPoint> = vec![
        //         DataPoint { data_point_id: 1, target: false, predicted: false, privileged_map: gender_pmap(), features: vec![1.0], timestamp: 0 },
        //         DataPoint { data_point_id: 2, target: false, predicted: false, privileged_map: gender_pmap(), features: vec![0.0], timestamp: 0 },
        //     ];
        //     let _eod = equal_opportunity_difference(&data, None);
        // }
    }

    #[cfg(test)]
    mod test_precision {
        use super::*;

        #[test]
        fn test_precision_basic() {
            let data = mock_data_points_precision();
            // TP=2, FP=2 => Precision=2/4=0.5
            let prec = precision(&data);
            assert!((prec - 0.5).abs() < 1e-6, "Precision should be 0.5");
        }

        #[test]
        #[should_panic(expected = "trap should only be called inside canisters.")]
        fn test_precision_no_positive_predictions() {
            let data = vec![
                DataPoint { data_point_id: 1, target: true, predicted: false, privileged_map: gender_pmap(), features: vec![0.0], timestamp: 0 },
                DataPoint { data_point_id: 2, target: false, predicted: false, privileged_map: gender_pmap(), features: vec![0.0], timestamp: 0 },
            ];
            let _prec = precision(&data);
        }
    }

    #[cfg(test)]
    mod test_recall {
        use super::*;

        #[test]
        fn test_recall_basic() {
            let data = mock_data_points_recall();
            // TP=2, FN=2 => Recall=2/4=0.5
            let rec = recall(&data);
            assert!((rec - 0.5).abs() < 1e-6, "Recall should be 0.5");
        }

        #[test]
        #[should_panic(expected = "trap should only be called inside canisters.")]
        fn test_recall_no_actual_positives() {
            let data = vec![
                DataPoint { data_point_id: 1, target: false, predicted: false, privileged_map: gender_pmap(), features: vec![0.0], timestamp: 0 },
                DataPoint { data_point_id: 2, target: false, predicted: false, privileged_map: gender_pmap(), features: vec![0.0], timestamp: 0 },
            ];
            let _rec = recall(&data);
        }
    }
}

