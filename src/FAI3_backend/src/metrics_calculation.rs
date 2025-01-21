use crate::{check_cycles_before_action, DataPoint, Model, User, USERS};
use candid::Principal;

use std::cell::RefCell;
use std::collections::HashMap;
use std::hash::Hash;

#[ic_cdk::update]
pub(crate) fn calculate_statistical_parity_difference(model_id: u128) -> HashMap<String, f32> {
    check_cycles_before_action();
    USERS.with(|users: &RefCell<HashMap<Principal, User>>| {
        let mut users: std::cell::RefMut<'_, HashMap<Principal, User>> = users.borrow_mut();
        let user: &mut User = users
            .get_mut(&ic_cdk::api::caller())
            .expect("User not found");
        let model: &mut Model = user
            .models
            .get_mut(&model_id)
            .expect("Model not found or not owned by user");

        if model.user_id != ic_cdk::api::caller() {
            ic_cdk::api::trap("Unauthorized: You are not the owner of this model");
        }

        let (
            privileged_count,
            unprivileged_count,
            privileged_positive_count,
            unprivileged_positive_count,
        ) = calculate_group_counts(&model.data_points);

        // Handle empty group scenario
        if privileged_count.len() == 0 || unprivileged_count.len() == 0 {
            ic_cdk::api::trap("Cannot calculate statistical parity difference: One of the groups has no data points.");
        }

        let mut result = HashMap::new();

        for (key, _) in &privileged_count {
            // let privileged_probability: f32 = *privileged_positive_count.get(key).unwrap_or(&0) as f32 / *privileged_count.get(key).unwrap_or(&0) as f32;
            // let unprivileged_probability: f32 = *unprivileged_positive_count.get(key).unwrap_or(&0) as f32 / *unprivileged_count.get(key).unwrap_or(&0) as f32;

            // let diff = unprivileged_probability - privileged_probability;

            let positives = *unprivileged_positive_count.get(key).unwrap_or(&0) as f32 / *privileged_positive_count.get(key).unwrap_or(&0) as f32;
            let instances = *unprivileged_count.get(key).unwrap_or(&0) as f32 / *privileged_count.get(key).unwrap_or(&0) as f32;

            let diff = positives - instances;

            result.insert(key.clone(), diff);
        }

        // let privileged_probability: f32 =
        //     privileged_positive_count as f32 / privileged_count as f32;
        // let unprivileged_probability: f32 =
        //     unprivileged_positive_count as f32 / unprivileged_count as f32;

        // let result: f32 = unprivileged_probability - privileged_probability;
        
        model.metrics.statistical_parity_difference = Some(result.clone());

        // Update timestamp after calculation
        model.metrics.timestamp = ic_cdk::api::time();

        result.clone()
    })
}

#[ic_cdk::update]
pub(crate) fn calculate_disparate_impact(model_id: u128) -> HashMap<String, f32> {
    check_cycles_before_action();
    USERS.with(|users: &RefCell<HashMap<Principal, User>>| {
        let mut users: std::cell::RefMut<'_, HashMap<Principal, User>> = users.borrow_mut();
        let user: &mut User = users
            .get_mut(&ic_cdk::api::caller())
            .expect("User not found");
        let model: &mut Model = user
            .models
            .get_mut(&model_id)
            .expect("Model not found or not owned by user");

        if model.user_id != ic_cdk::api::caller() {
            ic_cdk::api::trap("Unauthorized: You are not the owner of this model");
        }

        let (
            privileged_count,
            unprivileged_count,
            privileged_positive_count,
            unprivileged_positive_count,
        ) = calculate_group_counts(&model.data_points);

        if privileged_count.len() == 0 || unprivileged_count.len() == 0 {
            ic_cdk::api::trap("Cannot calculate statistical parity difference: One of the groups has no data points.");
        }

        let mut result = HashMap::new();

        for (key, _) in &privileged_count {
            let privileged_probability: f32 =
                *privileged_positive_count.get(key).unwrap_or(&0) as f32 / *privileged_count.get(key).unwrap_or(&0) as f32;
            let unprivileged_probability: f32 =
                *unprivileged_positive_count.get(key).unwrap_or(&0) as f32 / *unprivileged_count.get(key).unwrap_or(&0) as f32;

            let diff = unprivileged_probability / privileged_probability;
            result.insert(key.clone(), diff);
        }


        // let privileged_probability: f32 =
        //     privileged_positive_count as f32 / privileged_count as f32;
        // let unprivileged_probability: f32 =
        //     unprivileged_positive_count as f32 / unprivileged_count as f32;

        // assert!(
        //     privileged_probability > 0.0,
        //     "Privileged group has no positive outcomes"
        // );

        // let result: f32 = unprivileged_probability / privileged_probability;
        model.metrics.disparate_impact = Some(result.clone());

        // Update timestamp after calculation
        model.metrics.timestamp = ic_cdk::api::time();

        result.clone()
    })
}

#[ic_cdk::update]
pub(crate) fn calculate_average_odds_difference(model_id: u128) -> HashMap<String, f32> {
    check_cycles_before_action();
    USERS.with(|users: &RefCell<HashMap<Principal, User>>| {
        let mut users: std::cell::RefMut<'_, HashMap<Principal, User>> = users.borrow_mut();
        let user: &mut User = users
            .get_mut(&ic_cdk::api::caller())
            .expect("User not found");
        let model: &mut Model = user
            .models
            .get_mut(&model_id)
            .expect("Model not found or not owned by user");

        if model.user_id != ic_cdk::api::caller() {
            ic_cdk::api::trap("Unauthorized: You are not the owner of this model");
        }

        let (
            privileged_tp,
            privileged_fp,
            privileged_tn,
            privileged_fn,
            unprivileged_tp,
            unprivileged_fp,
            unprivileged_tn,
            unprivileged_fn,
        ) = calculate_confusion_matrix(&model.data_points);

        let mut result = HashMap::new();

        for (key, _) in &privileged_tp {
            let privileged_positive_total = *privileged_tp.get(key).unwrap_or(&0) + *privileged_fn.get(key).unwrap_or(&0);
            let unprivileged_positive_total = *unprivileged_tp.get(key).unwrap_or(&0) + *unprivileged_fn.get(key).unwrap_or(&0);
            let privileged_negative_total = *privileged_fp.get(key).unwrap_or(&0) + *privileged_tn.get(key).unwrap_or(&0);
            let unprivileged_negative_total = *unprivileged_fp.get(key).unwrap_or(&0) + *unprivileged_tn.get(key).unwrap_or(&0);

            // if privileged_positive_total == 0 || unprivileged_positive_total == 0 || privileged_negative_total == 0 || unprivileged_negative_total == 0 {
            //     ic_cdk::api::trap("Cannot calculate average odds difference: One of the groups has no data points or no positives/negatives.");
            // }

            let privileged_tpr: f32 = *privileged_tp.get(key).unwrap_or(&0) as f32 / (privileged_positive_total + 1) as f32;
            let unprivileged_tpr: f32 = *unprivileged_tp.get(key).unwrap_or(&0) as f32 / (unprivileged_positive_total + 1) as f32;
            let privileged_fpr: f32 = *privileged_fp.get(key).unwrap_or(&0) as f32 / (privileged_negative_total + 1) as f32;
            let unprivileged_fpr: f32 = *unprivileged_fp.get(key).unwrap_or(&0) as f32 / (unprivileged_negative_total + 1) as f32;

            let diff = ((unprivileged_fpr - privileged_fpr) + (unprivileged_tpr - privileged_tpr)) / 2.0;
            result.insert(key.clone(), diff);
        }

        // let privileged_positive_total = privileged_tp + privileged_fn;
        // let unprivileged_positive_total = unprivileged_tp + unprivileged_fn;
        // let privileged_negative_total = privileged_fp + privileged_tn;
        // let unprivileged_negative_total = unprivileged_fp + unprivileged_tn;

        //if privileged_positive_total == 0 || unprivileged_positive_total == 0 || privileged_negative_total == 0 || unprivileged_negative_total == 0 {
        //    ic_cdk::api::trap("Cannot calculate average odds difference: One of the groups has no data points or no positives/negatives.");
        //}

        // let privileged_tpr: f32 = privileged_tp as f32 / (privileged_tp + privileged_fn + 1) as f32;
        // let unprivileged_tpr: f32 =
        //     unprivileged_tp as f32 / (unprivileged_tp + unprivileged_fn + 1) as f32;
        // let privileged_fpr: f32 = privileged_fp as f32 / (privileged_fp + privileged_tn + 1) as f32;
        // let unprivileged_fpr: f32 =
        //     unprivileged_fp as f32 / (unprivileged_fp + unprivileged_tn + 1) as f32;

        // let result: f32 =
        //     ((unprivileged_fpr - privileged_fpr) + (unprivileged_tpr - privileged_tpr)) / 2.0;
        model.metrics.average_odds_difference = Some(result.clone());

        // Update timestamp after calculation
        model.metrics.timestamp = ic_cdk::api::time();

        result
    })
}

#[ic_cdk::update]
pub(crate) fn calculate_equal_opportunity_difference(model_id: u128) -> HashMap<String, f32> {
    check_cycles_before_action();
    USERS.with(|users| {
        let mut users = users.borrow_mut();
        let user = users
            .get_mut(&ic_cdk::api::caller())
            .expect("User not found");
        let model = user
            .models
            .get_mut(&model_id)
            .expect("Model not found or not owned by user");

        if model.user_id != ic_cdk::api::caller() {
            ic_cdk::api::trap("Unauthorized: You are not the owner of this model");
        }

        let (privileged_tp, privileged_fn, unprivileged_tp, unprivileged_fn) =
            calculate_true_positive_false_negative(&model.data_points);

        let mut result = HashMap::new();

        for (key, _) in &privileged_tp {
            let privileged_positive_total = *privileged_tp.get(key).unwrap_or(&0) + *privileged_fn.get(key).unwrap_or(&0);
            let unprivileged_positive_total = *unprivileged_tp.get(key).unwrap_or(&0) + *unprivileged_fn.get(key).unwrap_or(&0);

            if privileged_positive_total == 0 || unprivileged_positive_total == 0 {
                ic_cdk::api::trap("Cannot calculate equal opportunity difference: One of the groups has no positive data points.");
            }

            let privileged_tpr = *privileged_tp.get(key).unwrap_or(&0) as f32 / (privileged_positive_total + 1) as f32;
            let unprivileged_tpr = *unprivileged_tp.get(key).unwrap_or(&0) as f32 / (unprivileged_positive_total + 1) as f32;

            let diff = unprivileged_tpr - privileged_tpr;
            result.insert(key.clone(), diff);
        }

        // let privileged_positive_total = privileged_tp + privileged_fn;
        // let unprivileged_positive_total = unprivileged_tp + unprivileged_fn;

        //if privileged_positive_total == 0 || unprivileged_positive_total == 0 {
        //    ic_cdk::api::trap("Cannot calculate equal opportunity difference: One of the groups has no positive data points.");
        //}

        // let privileged_tpr = privileged_tp as f32 / (privileged_positive_total + 1) as f32;
        // let unprivileged_tpr = unprivileged_tp as f32 / (unprivileged_positive_total + 1) as f32;

        // let result = unprivileged_tpr - privileged_tpr;
        model.metrics.equal_opportunity_difference = Some(result.clone());
        model.metrics.timestamp = ic_cdk::api::time();

        result
    })
}

#[ic_cdk::update]
pub(crate) fn calculate_accuracy(model_id: u128) -> f32 {
    check_cycles_before_action();
    USERS.with(|users| {
        let mut users = users.borrow_mut();
        let user = users
            .get_mut(&ic_cdk::api::caller())
            .expect("User not found");
        let model = user
            .models
            .get_mut(&model_id)
            .expect("Model not found or not owned by user");

        if model.user_id != ic_cdk::api::caller() {
            ic_cdk::api::trap("Unauthorized");
        }

        let (tp, tn, fp, fn_) = calculate_overall_confusion_matrix(&model.data_points);
        let total = tp + tn + fp + fn_;
        if total == 0 {
            ic_cdk::api::trap("No data points to calculate accuracy");
        }

        let accuracy = (tp + tn) as f32 / total as f32;
        model.metrics.accuracy = Some(accuracy);
        model.metrics.timestamp = ic_cdk::api::time();
        accuracy
    })
}

#[ic_cdk::update]
pub(crate) fn calculate_precision(model_id: u128) -> f32 {
    check_cycles_before_action();
    USERS.with(|users| {
        let mut users = users.borrow_mut();
        let user = users
            .get_mut(&ic_cdk::api::caller())
            .expect("User not found");
        let model = user
            .models
            .get_mut(&model_id)
            .expect("Model not found or not owned by user");

        if model.user_id != ic_cdk::api::caller() {
            ic_cdk::api::trap("Unauthorized");
        }

        let (tp, _, fp, _) = calculate_overall_confusion_matrix(&model.data_points);
        let denominator = tp + fp;
        if denominator == 0 {
            ic_cdk::api::trap("No positive predictions for precision");
        }

        let precision = tp as f32 / denominator as f32;
        model.metrics.precision = Some(precision);
        model.metrics.timestamp = ic_cdk::api::time();
        precision
    })
}

#[ic_cdk::update]
pub(crate) fn calculate_recall(model_id: u128) -> f32 {
    check_cycles_before_action();
    USERS.with(|users| {
        let mut users = users.borrow_mut();
        let user = users
            .get_mut(&ic_cdk::api::caller())
            .expect("User not found");
        let model = user
            .models
            .get_mut(&model_id)
            .expect("Model not found or not owned by user");

        // Update the timestamp again here if needed, or rely on the last calculated metric

        if model.user_id != ic_cdk::api::caller() {
            ic_cdk::api::trap("Unauthorized");
        }

        let (tp, _, _, fn_) = calculate_overall_confusion_matrix(&model.data_points);
        let denominator = tp + fn_;
        if denominator == 0 {
            ic_cdk::api::trap("No actual positives for recall");
        }

        let recall = tp as f32 / denominator as f32;
        model.metrics.recall = Some(recall);
        model.metrics.timestamp = ic_cdk::api::time();

        // Push the updated metrics to the history
        recall
    })
}

#[ic_cdk::update]
pub(crate) fn calculate_all_metrics(model_id: u128) -> (HashMap<String, f32>, HashMap<String, f32>, HashMap<String, f32>, HashMap<String, f32>, f32, f32, f32) {
    let spd = calculate_statistical_parity_difference(model_id);
    let di = calculate_disparate_impact(model_id);
    let aod = calculate_average_odds_difference(model_id);
    let eod = calculate_equal_opportunity_difference(model_id);
    let acc = calculate_accuracy(model_id);
    let prec = calculate_precision(model_id);
    let rec = calculate_recall(model_id);

    USERS.with(|users| {
        let mut users = users.borrow_mut();
        let user = users
            .get_mut(&ic_cdk::api::caller())
            .expect("User not found");
        let model = user
            .models
            .get_mut(&model_id)
            .expect("Model not found or not owned by user");
        model.metrics.timestamp = ic_cdk::api::time();
        model.metrics_history.push(model.metrics.clone());
    });

    (spd, di, aod, eod, acc, prec, rec)
}

pub(crate) fn calculate_group_counts(
    data_points: &Vec<DataPoint>,
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

    for point in data_points {
        let features_list = point.features.clone();

        for entry in &point.privileged_map {
            let vairable_name = entry.0;
            let variable_index = entry.1;

            if features_list[*variable_index as usize] > 0.0 {
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

        // if point.privileged {
        //     privileged_count += 1;
        //     if point.predicted {
        //         privileged_positive_count += 1;
        //     }
        // } else {
        //     unprivileged_count += 1;
        //     if point.predicted {
        //         unprivileged_positive_count += 1;
        //     }
        // }
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
) -> (HashMap<String, u128>, HashMap<String, u128>, HashMap<String, u128>, HashMap<String, u128>, HashMap<String, u128>, HashMap<String, u128>, HashMap<String, u128>, HashMap<String, u128>) {
    let (mut privileged_tp, mut privileged_fp, mut privileged_tn, mut privileged_fn) = (HashMap::new(), HashMap::new(), HashMap::new(), HashMap::new());
    let (unprivileged_tp, unprivileged_fp, unprivileged_tn, unprivileged_fn) =
        (HashMap::new(), HashMap::new(), HashMap::new(), HashMap::new());

    for point in data_points {
        let features_list = point.features.clone();
        for entry in point.privileged_map.iter() {
            let vairable_name = entry.0;
            let variable_index = entry.1;

            if features_list[*variable_index as usize] <= 0.0 {
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
                    privileged_fp
                        .entry(vairable_name.clone())
                        .and_modify(|e| *e += 1)
                        .or_insert(1);
                }
                (false, false) => {
                    privileged_tn
                        .entry(vairable_name.clone())
                        .and_modify(|e| *e += 1)
                        .or_insert(1);
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
) -> (HashMap<String, u128>, HashMap<String, u128>, HashMap<String, u128>, HashMap<String, u128>) {
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
