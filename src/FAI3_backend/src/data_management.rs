use crate::{
    check_cycles_before_action, get_model, is_owner, DataPoint, Model, User, MODELS,
    NEXT_DATA_POINT_ID,
};
use candid::{CandidType, Deserialize, Principal};
use std::cell::RefCell;
use std::collections::HashMap;

#[derive(CandidType, Deserialize)]
pub(crate) struct KeyValuePair {
    key: String,
    value: u128,
}

#[ic_cdk::update]
pub fn add_dataset(
    model_id: u128,
    features: Vec<Vec<f64>>,
    labels: Vec<bool>,
    predictions: Vec<bool>,
    privileged: Vec<KeyValuePair>,
) {
    check_cycles_before_action();

    // Verify that all columns have consistent lengths (unchanged)
    let data_length = labels.len();
    if predictions.len() != data_length {
        ic_cdk::api::trap("Error: Lengths of labels and predictions must be equal.");
    }
    for feature_column in &features {
        if feature_column.len() != data_length {
            ic_cdk::api::trap("Error: All feature columns must have the same length as labels.");
        }
    }

    let caller: Principal = ic_cdk::api::caller();
    let timestamp: u64 = ic_cdk::api::time();

    // let privileged_map: HashMap<String, u128> = privileged_labels.iter().enumerate().map(|(i, label)| {
    //     (label.clone(), privilege_indices[i])
    // }).collect();

    let privileged_map: HashMap<String, u128> = privileged
        .iter()
        .map(|pair| (pair.key.clone(), pair.value as u128))
        .collect();

    MODELS.with(|models| {
        let mut models = models.borrow_mut();
        let mut model = models.get(&model_id).expect("Model not found");

        is_owner(&model, caller);

        NEXT_DATA_POINT_ID.with(|id| {
            let mut next_data_point_id = id.borrow_mut();
            for i in 0..data_length {
                let mut feature_vector = Vec::new();
                for feature_column in &features {
                    feature_vector.push(feature_column[i]);
                }

                // Determine privileged status using u64 and casting to usize
                // let mut privileged = false;
                // for &index in &privilege_indices {
                //     let idx = index as usize;
                //     if idx < feature_vector.len() && feature_vector[idx] > 0.0 {
                //         privileged = true;
                //         break;
                //     }
                // }

                let data_point = DataPoint {
                    data_point_id: *next_data_point_id.get(),
                    target: labels[i],
                    privileged_map: privileged_map.clone(),
                    predicted: predictions[i],
                    features: feature_vector.clone(),
                    timestamp,
                };

                model.data_points.push(data_point);
                models.insert(model_id, model.clone());
                let current_id = *next_data_point_id.get();
                next_data_point_id.set(current_id + 1).unwrap();
            }
        });
    });
}

#[ic_cdk::update]
pub fn add_data_point(
    model_id: u128,
    target: bool,
    privilege_indices: Vec<u128>,
    privileged_labels: Vec<String>,
    predicted: bool,
    features: Vec<f64>,
) {
    check_cycles_before_action();
    let caller: Principal = ic_cdk::api::caller();
    let timestamp: u64 = ic_cdk::api::time();

    let privileged_map: HashMap<String, u128> = privileged_labels
        .iter()
        .enumerate()
        .map(|(i, label)| (label.clone(), privilege_indices[i]))
        .collect();

    MODELS.with(|models| {
        let mut models = models.borrow_mut();
        let mut model = models.get(&model_id).expect("Model not found");

        is_owner(&model, caller);

        NEXT_DATA_POINT_ID.with(|next_data_point_id| {
            let data_point_id = *next_data_point_id.borrow().get();
    
            let data_point: DataPoint = DataPoint {
                data_point_id,
                target,
                privileged_map: privileged_map.clone(),
                predicted,
                features,
                timestamp,
            };
    
            model.data_points.push(data_point);
            models.insert(model_id, model.clone());
            next_data_point_id.borrow_mut().set(next_data_point_id.borrow().get() + 1).unwrap()
        });
    });
}

#[ic_cdk::update]
pub fn delete_data_point(model_id: u128, data_point_id: u128) {
    check_cycles_before_action();
    let caller: Principal = ic_cdk::api::caller();

    MODELS.with(|models| {
        let models = models.borrow_mut();
        let mut model = models.get(&model_id).expect("Model not found");

        is_owner(&model, caller);

        let data_point_index = model
            .data_points
            .iter()
            .position(|d| d.data_point_id == data_point_id)
            .expect("Data point not found");
        model.data_points.remove(data_point_index);
    });
}
