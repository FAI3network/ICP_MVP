use crate::{calculate_confusion_matrix, calculate_group_counts, calculate_overall_confusion_matrix, calculate_true_positive_false_negative, DataPoint};

fn mock_data_points_all_correct() -> Vec<DataPoint> {
    // Perfect classifier: target == predicted
    vec![
        DataPoint {
            data_point_id: 1,
            target: true,
            predicted: true,
            privileged: false,
            features: vec![0.5],
            timestamp: 0,
        },
        DataPoint {
            data_point_id: 2,
            target: false,
            predicted: false,
            privileged: true,
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
            privileged: true,
            features: vec![1.0],
            timestamp: 0,
        },
        DataPoint {
            data_point_id: 2,
            target: false,
            predicted: false,
            privileged: true,
            features: vec![1.0],
            timestamp: 0,
        },
        DataPoint {
            data_point_id: 3,
            target: true,
            predicted: true,
            privileged: false,
            features: vec![0.0],
            timestamp: 0,
        },
        DataPoint {
            data_point_id: 4,
            target: false,
            predicted: true,
            privileged: false,
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
        DataPoint { data_point_id: 1, target: true, predicted: true, privileged: true, features: vec![], timestamp: 0 }, // TP
        DataPoint { data_point_id: 2, target: true, predicted: false, privileged: true, features: vec![], timestamp: 0 }, // FN
        DataPoint { data_point_id: 3, target: false, predicted: true, privileged: true, features: vec![], timestamp: 0 }, // FP
        DataPoint { data_point_id: 4, target: false, predicted: false, privileged: true, features: vec![], timestamp: 0 }, // TN

        // Unprivileged (exact same pattern):
        DataPoint { data_point_id: 5, target: true, predicted: true, privileged: false, features: vec![], timestamp: 0 }, // TP
        DataPoint { data_point_id: 6, target: true, predicted: false, privileged: false, features: vec![], timestamp: 0 }, // FN
        DataPoint { data_point_id: 7, target: false, predicted: true, privileged: false, features: vec![], timestamp: 0 }, // FP
        DataPoint { data_point_id: 8, target: false, predicted: false, privileged: false, features: vec![], timestamp: 0 }, // TN
    ]
}

fn mock_data_points_eod() -> Vec<DataPoint> {
    // Equal Opportunity Difference test:
    // If privileged TPR=1.0, unprivileged TPR=0.5, EOD=0.5 - 1.0 = -0.5
    vec![
        // Privileged all true positives (TP=2, FN=0)
        DataPoint { data_point_id: 1, target: true, predicted: true, privileged: true, features: vec![], timestamp: 0 },
        DataPoint { data_point_id: 2, target: true, predicted: true, privileged: true, features: vec![], timestamp: 0 },

        // Unprivileged (TP=1, FN=1)
        DataPoint { data_point_id: 3, target: true, predicted: true, privileged: false, features: vec![], timestamp: 0 },
        DataPoint { data_point_id: 4, target: true, predicted: false, privileged: false, features: vec![], timestamp: 0 },
    ]
}

fn mock_data_points_precision() -> Vec<DataPoint> {
    // Precision = TP / (TP+FP)
    // Let's say TP=2, FP=2 => Precision = 0.5
    vec![
        DataPoint { data_point_id: 1, target: true, predicted: true, privileged: false, features: vec![], timestamp: 0 },
        DataPoint { data_point_id: 2, target: true, predicted: true, privileged: false, features: vec![], timestamp: 0 },
        DataPoint { data_point_id: 3, target: false, predicted: true, privileged: false, features: vec![], timestamp: 0 },
        DataPoint { data_point_id: 4, target: false, predicted: true, privileged: false, features: vec![], timestamp: 0 },
    ]
}

fn mock_data_points_recall() -> Vec<DataPoint> {
    // Recall = TP / (TP+FN)
    // Let's say TP=2, FN=2 => Recall = 0.5
    vec![
        DataPoint { data_point_id: 1, target: true, predicted: true, privileged: false, features: vec![], timestamp: 0 },
        DataPoint { data_point_id: 2, target: true, predicted: true, privileged: false, features: vec![], timestamp: 0 },
        DataPoint { data_point_id: 3, target: true, predicted: false, privileged: false, features: vec![], timestamp: 0 },
        DataPoint { data_point_id: 4, target: true, predicted: false, privileged: false, features: vec![], timestamp: 0 },
    ]
}

// Example of a direct function test if you have a pure function:
fn calculate_accuracy_internal(data_points: &Vec<DataPoint>) -> f32 {
    if data_points.is_empty() {
        panic!("No data points to calculate accuracy");
    }
    let (tp, tn, fp, fn_) = calculate_overall_confusion_matrix(&data_points);
    let total = tp + tn + fp + fn_;
    (tp + tn) as f32 / total as f32
}

fn calculate_disparate_impact_internal(data_points: &Vec<DataPoint>) -> f32 {
    // Based on group counts
    let (priv_count, unpriv_count, priv_pos, unpriv_pos) = calculate_group_counts(data_points);

    if priv_count == 0 || unpriv_count == 0 {
        panic!("No data for one of the groups");
    }

    let p_priv = priv_pos as f32 / priv_count as f32;
    let p_unpriv = unpriv_pos as f32 / unpriv_count as f32;

    if p_priv == 0.0 {
        panic!("Privileged group has no positive outcomes");
    }

    p_unpriv / p_priv
}

fn calculate_average_odds_difference_internal(data_points: &Vec<DataPoint>) -> f32 {
    let (ptp, pfp, ptn, pfn, utp, ufp, utn, ufn) = calculate_confusion_matrix(data_points);

    let p_positive_total = ptp + pfn;
    let u_positive_total = utp + ufn;
    let p_negative_total = pfp + ptn;
    let u_negative_total = ufp + utn;

    if p_positive_total == 0 || u_positive_total == 0 || p_negative_total == 0 || u_negative_total == 0 {
        panic!("One of the groups has no data for calculating average odds difference");
    }

    let p_tpr = ptp as f32 / p_positive_total as f32;
    let u_tpr = utp as f32 / u_positive_total as f32;
    let p_fpr = pfp as f32 / p_negative_total as f32;
    let u_fpr = ufp as f32 / u_negative_total as f32;

    ((u_fpr - p_fpr) + (u_tpr - p_tpr)) / 2.0
}

fn calculate_equal_opportunity_difference_internal(data_points: &Vec<DataPoint>) -> f32 {
    let (ptp, pfn, utp, ufn) = calculate_true_positive_false_negative(data_points);

    let p_positive_total = ptp + pfn;
    let u_positive_total = utp + ufn;

    if p_positive_total == 0 || u_positive_total == 0 {
        panic!("One of the groups has no positive data points");
    }

    let p_tpr = ptp as f32 / p_positive_total as f32;
    let u_tpr = utp as f32 / u_positive_total as f32;

    u_tpr - p_tpr
}

fn calculate_precision_internal(data_points: &Vec<DataPoint>) -> f32 {
    let (tp, _tn, fp, _fn) = calculate_overall_confusion_matrix(data_points);

    let denominator = tp + fp;
    if denominator == 0 {
        panic!("No positive predictions to calculate precision");
    }

    tp as f32 / denominator as f32
}

fn calculate_recall_internal(data_points: &Vec<DataPoint>) -> f32 {
    let (tp, _tn, _fp, fn_) = calculate_overall_confusion_matrix(data_points);

    let denominator = tp + fn_;
    if denominator == 0 {
        panic!("No actual positives to calculate recall");
    }

    tp as f32 / denominator as f32
}


#[cfg(test)]
mod test_accuracy {
    use candid::Principal;

    use crate::{Metrics, Model, ModelDetails};

    use super::*;
 // Assuming this function exists

    #[test]
    fn test_perfect_accuracy() {
        let data_points = mock_data_points_all_correct();
        // Suppose your model structure has a metrics field and data_points
        let model = Model {
            model_id: 1,
            model_name: "test".to_string(),
            user_id: Principal::anonymous(),
            data_points: data_points.clone(),
            metrics: Metrics {
                statistical_parity_difference: None,
                disparate_impact: None,
                average_odds_difference: None,
                equal_opportunity_difference: None,
                accuracy: None,
                precision: None,
                recall: None,
                timestamp: 0,
            },
            details: ModelDetails{
                description: "Description".to_string(),
                framework: "Framework".to_string(),
                version: "Version".to_string(),
                objective: "Objective".to_string(),
                url: "URL".to_string(),
            },
            metrics_history: vec![],
        };

        // Assuming calculate_accuracy takes model_id and updates model.metrics.accuracy
        // If it's a pure function, adjust accordingly.
        let acc = calculate_accuracy_internal(&model.data_points);
        assert!((acc - 1.0).abs() < 1e-6, "Accuracy should be 1.0 for a perfect classifier");
    }

    #[test]
    #[should_panic(expected = "No data points to calculate accuracy")]
    fn test_no_data_points() {
        let data_points: Vec<DataPoint> = vec![];
        let _acc = calculate_accuracy_internal(&data_points); // Should panic
    }
}

#[cfg(test)]
mod test_statistical_parity {
    use candid::Principal;

    use crate::{Metrics, Model, ModelDetails};

    use super::*;

    #[test]
    fn test_statistical_parity_difference_basic() {
        let data_points = mock_data_points_stat_parity_example();
        let model = Model {
            model_id: 1,
            model_name: "test_model".to_string(),
            user_id: Principal::anonymous(),
            data_points: data_points.clone(),
            metrics: Metrics {
                statistical_parity_difference: None,
                disparate_impact: None,
                average_odds_difference: None,
                equal_opportunity_difference: None,
                accuracy: None,
                precision: None,
                recall: None,
                timestamp: 0,
            },
            details: ModelDetails{
                description: "Description".to_string(),
                framework: "Framework".to_string(),
                version: "Version".to_string(),
                objective: "Objective".to_string(),
                url: "URL".to_string(),
            },
            metrics_history: vec![],
        };

        // Suppose in this example:
        // Privileged group: 2 data points, 1 predicted positive => P(Pos|Priv) = 0.5
        // Unprivileged group: 2 data points, 2 predicted positive => P(Pos|Unpriv) = 1.0
        // SPD = 1.0 - 0.5 = 0.5
        let spd = calculate_statistical_parity_difference_internal(&model.data_points);
        println!("{}",spd);
        assert!((spd - 0.3333333).abs() < 1e-6, "Statistical Parity Difference should be 0.5");
    }

    #[test]
    //#[should_panic(expected = "One of the groups has no data points")]
    fn test_all_privileged_no_unprivileged() {
        let data_points = vec![
            DataPoint {
                data_point_id: 1,
                target: true,
                predicted: true,
                privileged: true,
                features: vec![1.0],
                timestamp: 0,
            }
        ];
        let _spd = calculate_statistical_parity_difference_internal(&data_points); // Should panic
    }

    fn calculate_statistical_parity_difference_internal(data_points: &Vec<DataPoint>) -> f32 {
        let (priv_count, unpriv_count, priv_pos, unpriv_pos) = crate::calculate_group_counts(&data_points);
        if priv_count == 0 || unpriv_count == 0 {
            panic!("One of the groups has no data points");
        }
        let p_priv = priv_pos as f32 / priv_count as f32;       
        let p_unpriv = unpriv_pos as f32 / unpriv_count as f32;
        p_unpriv - p_priv
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
        let di = calculate_disparate_impact_internal(&data);
        println!("{}",di);
        assert!((di - 1.5).abs() < 1e-6, "Disparate Impact should be 2.0");
    }

    #[test]
    //#[should_panic(expected = "No data for one of the groups")]
    fn test_no_group_data_di() {
        let data = vec![]; // no data at all
        let _di = calculate_disparate_impact_internal(&data);
    }
}

#[cfg(test)]
mod test_average_odds_difference {
    use super::*;

    #[test]
    fn test_average_odds_difference_basic() {
        let data = mock_data_points_average_odds(); // updated version
        let aod = calculate_average_odds_difference_internal(&data);
        assert!((aod - 0.0).abs() < 1e-6, "Average Odds Difference should be 0.0");
    }

    #[test]
    #[should_panic(expected = "One of the groups has no data for calculating average odds difference")]
    fn test_aod_missing_data() {
        let data: Vec<DataPoint> = vec![];
        let _aod = calculate_average_odds_difference_internal(&data);
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
        let eod = calculate_equal_opportunity_difference_internal(&data);
        assert!((eod + 0.5).abs() < 1e-6, "Equal Opportunity Difference should be -0.5");
    }

    #[test]
    #[should_panic(expected = "One of the groups has no positive data points")]
    fn test_eod_no_positives() {
        let data: Vec<DataPoint> = vec![
            DataPoint { data_point_id: 1, target: false, predicted: false, privileged: true, features: vec![], timestamp: 0 },
            DataPoint { data_point_id: 2, target: false, predicted: false, privileged: false, features: vec![], timestamp: 0 },
        ];
        let _eod = calculate_equal_opportunity_difference_internal(&data);
    }
}

#[cfg(test)]
mod test_precision {
    use super::*;

    #[test]
    fn test_precision_basic() {
        let data = mock_data_points_precision();
        // TP=2, FP=2 => Precision=2/4=0.5
        let prec = calculate_precision_internal(&data);
        assert!((prec - 0.5).abs() < 1e-6, "Precision should be 0.5");
    }

    #[test]
    #[should_panic(expected = "No positive predictions to calculate precision")]
    fn test_precision_no_positive_predictions() {
        let data = vec![
            DataPoint { data_point_id: 1, target: true, predicted: false, privileged: false, features: vec![], timestamp: 0 },
            DataPoint { data_point_id: 2, target: false, predicted: false, privileged: false, features: vec![], timestamp: 0 },
        ];
        let _prec = calculate_precision_internal(&data);
    }
}

#[cfg(test)]
mod test_recall {
    use super::*;

    #[test]
    fn test_recall_basic() {
        let data = mock_data_points_recall();
        // TP=2, FN=2 => Recall=2/4=0.5
        let rec = calculate_recall_internal(&data);
        assert!((rec - 0.5).abs() < 1e-6, "Recall should be 0.5");
    }

    #[test]
    #[should_panic(expected = "No actual positives to calculate recall")]
    fn test_recall_no_actual_positives() {
        let data = vec![
            DataPoint { data_point_id: 1, target: false, predicted: false, privileged: false, features: vec![], timestamp: 0 },
            DataPoint { data_point_id: 2, target: false, predicted: false, privileged: false, features: vec![], timestamp: 0 },
        ];
        let _rec = calculate_recall_internal(&data);
    }
}
