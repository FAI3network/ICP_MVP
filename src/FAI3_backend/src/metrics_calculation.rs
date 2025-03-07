use crate::types::PrivilegedIndex;
use crate::{check_cycles_before_action, is_owner, DataPoint, MODELS};

use core::time;
use std::collections::{HashMap, HashSet};
use ic_cdk::println;


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

        let latest_timestamp = model.data_points[model.data_points.len() - 1].timestamp;

        let relevant_data_points: Vec<DataPoint> = model.data_points
            .iter()
            .filter(|point| point.timestamp == latest_timestamp)
            .cloned()
            .collect();
        
        let (
            privileged_count,
            unprivileged_count,
            privileged_positive_count,
            unprivileged_positive_count,
        ) = calculate_group_counts(&relevant_data_points, privilieged_threshold);

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

        model.metrics.average_metrics.statistical_parity_difference = Some(average);

        model.metrics.statistical_parity_difference = Some(result.clone());

        // Update timestamp after calculation
        model.metrics.timestamp = ic_cdk::api::time();

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

        let latest_timestamp = model.data_points[model.data_points.len() - 1].timestamp;

        let relevant_data_points: Vec<DataPoint> = model.data_points
            .iter()
            .filter(|point| point.timestamp == latest_timestamp)
            .cloned()
            .collect();

        let (
            privileged_count,
            unprivileged_count,
            privileged_positive_count,
            unprivileged_positive_count,
        ) = calculate_group_counts(&relevant_data_points, privilieged_threshold);

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

        model.metrics.average_metrics.disparate_impact = Some(average);

        model.metrics.disparate_impact = Some(result.clone());

        // Update timestamp after calculation
        model.metrics.timestamp = ic_cdk::api::time();

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

    MODELS.with(|models| {
        let mut models = models.borrow_mut();
        let mut model = models.get(&model_id).expect("Model not found");

        is_owner(&model, caller);

        let latest_timestamp = model.data_points[model.data_points.len() - 1].timestamp;

        let relevant_data_points: Vec<DataPoint> = model.data_points
            .iter()
            .filter(|point| point.timestamp == latest_timestamp)
            .cloned()
            .collect();

        let (
            privileged_tp,
            privileged_fp,
            privileged_tn,
            privileged_fn,
            unprivileged_tp,
            unprivileged_fp,
            unprivileged_tn,
            unprivileged_fn,
        ) = calculate_confusion_matrix(&relevant_data_points, privilieged_threshold);

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

        let average: f32 = sum / length;

        model.metrics.average_metrics.average_odds_difference = Some(average);

        model.metrics.average_odds_difference = Some(result.clone());

        // Update timestamp after calculation
        model.metrics.timestamp = ic_cdk::api::time();

        models.insert(model_id, model.clone());

        result
    })
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

        let latest_timestamp = model.data_points[model.data_points.len() - 1].timestamp;

        let relevant_data_points: Vec<DataPoint> = model.data_points
            .iter()
            .filter(|point| point.timestamp == latest_timestamp)
            .cloned()
            .collect();

        let mut count_pred_label_unprivileged = HashMap::new();
        let mut count_pred_label_privileged = HashMap::new();
        let mut count_label_unprivileged = HashMap::new();
        let mut count_label_privileged = HashMap::new();

        let threshold_map = if privilieged_threshold.is_some() {
            privilieged_threshold.unwrap()
        } else {
            calculate_medians(&model.data_points)
        };

        for point in &relevant_data_points {
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
                    / (*count_label_unprivileged.get(key).unwrap_or(&0.0) + 1.0);
            let prob_pred_label_privileged = *count_pred_label_privileged.get(key).unwrap_or(&0.0)
                / (*count_label_privileged.get(key).unwrap_or(&0.0) + 1.0);

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

        model.metrics.average_metrics.equal_opportunity_difference = Some(average);

        model.metrics.equal_opportunity_difference = Some(result.clone());
        model.metrics.timestamp = ic_cdk::api::time();

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

        let latest_timestamp = model.data_points[model.data_points.len() - 1].timestamp;

        let relevant_data_points: Vec<DataPoint> = model.data_points
            .iter()
            .filter(|point| point.timestamp == latest_timestamp)
            .cloned()
            .collect();

        let (tp, tn, fp, fn_) = calculate_overall_confusion_matrix(&relevant_data_points);
        let total = tp + tn + fp + fn_;
        if total == 0 {
            ic_cdk::api::trap("No data points to calculate accuracy");
        }

        let accuracy = (tp + tn) as f32 / total as f32;
        model.metrics.accuracy = Some(accuracy);
        model.metrics.timestamp = ic_cdk::api::time();
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

        let latest_timestamp = model.data_points[model.data_points.len() - 1].timestamp;

        let relevant_data_points: Vec<DataPoint> = model.data_points
            .iter()
            .filter(|point| point.timestamp == latest_timestamp)
            .cloned()
            .collect();

        let (tp, _, fp, _) = calculate_overall_confusion_matrix(&relevant_data_points);
        let denominator = tp + fp;
        if denominator == 0 {
            ic_cdk::api::trap("No positive predictions for precision");
        }

        let precision = tp as f32 / denominator as f32;
        model.metrics.precision = Some(precision);
        model.metrics.timestamp = ic_cdk::api::time();
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

        let latest_timestamp = model.data_points[model.data_points.len() - 1].timestamp;

        let relevant_data_points: Vec<DataPoint> = model.data_points
            .iter()
            .filter(|point| point.timestamp == latest_timestamp)
            .cloned()
            .collect();

        let (tp, _, _, fn_) = calculate_overall_confusion_matrix(&relevant_data_points);
        let denominator = tp + fn_;
        if denominator == 0 {
            ic_cdk::api::trap("No actual positives for recall");
        }

        let recall = tp as f32 / denominator as f32;
        model.metrics.recall = Some(recall);
        model.metrics.timestamp = ic_cdk::api::time();
        models.insert(model_id, model.clone());

        // Push the updated metrics to the history
        recall
    })
}

#[ic_cdk::update]
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

        model.metrics.timestamp = ic_cdk::api::time();
        model.metrics_history.push(model.metrics.clone());
        models.insert(model_id, model.clone());
    });

    (spd, di, aod, eod, acc, prec, rec)
}

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
        // match (point.privileged, point.target, point.predicted) {
        //     (true, true, true) => privileged_tp += 1,
        //     (true, true, false) => privileged_fn += 1,
        //     (true, false, true) => privileged_fp += 1,
        //     (true, false, false) => privileged_tn += 1,
        //     (false, true, true) => unprivileged_tp += 1,
        //     (false, true, false) => unprivileged_fn += 1,
        //     (false, false, true) => unprivileged_fp += 1,
        //     (false, false, false) => unprivileged_tn += 1,
        // }
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

pub(crate) fn calculate_true_positive_false_negative(
    data_points: &Vec<DataPoint>,
) -> (
    HashMap<String, u128>,
    HashMap<String, u128>,
    HashMap<String, u128>,
    HashMap<String, u128>,
) {
    let (mut privileged_tp, mut privileged_fn) = (HashMap::new(), HashMap::new());
    let (mut unprivileged_tp, mut unprivileged_fn) = (HashMap::new(), HashMap::new());

    for point in data_points {
        for entry in point.privileged_map.iter() {
            let vairable_name = entry.0;
            let variable_index = entry.1;

            if point.features[*variable_index as usize] <= 0.0 {
                continue;
            }

            match (point.target, point.predicted) {
                (true, true) => {
                    privileged_tp
                        .entry(vairable_name.clone())
                        .and_modify(|e| *e += 1)
                        .or_insert(1);
                }
                (true, false) => {
                    privileged_fn
                        .entry(vairable_name.clone())
                        .and_modify(|e| *e += 1)
                        .or_insert(1);
                }
                (false, true) => {
                    unprivileged_tp
                        .entry(vairable_name.clone())
                        .and_modify(|e| *e += 1)
                        .or_insert(1);
                }
                (false, false) => {
                    unprivileged_fn
                        .entry(vairable_name.clone())
                        .and_modify(|e| *e += 1)
                        .or_insert(1);
                }
            }
        }

        // match (point.privileged, point.target, point.predicted) {
        //     (true, true, true) => privileged_tp += 1,
        //     (true, true, false) => privileged_fn += 1,
        //     (false, true, true) => unprivileged_tp += 1,
        //     (false, true, false) => unprivileged_fn += 1,
        //     _ => {}
        // }
    }

    (
        privileged_tp,
        privileged_fn,
        unprivileged_tp,
        unprivileged_fn,
    )
}

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
