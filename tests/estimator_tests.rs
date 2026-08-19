use relay_curve_estimator::curves::{CurveDefinition, CurveStandard, CurveType};
use relay_curve_estimator::estimator::{RelayEstimator, TestPoint};

#[test]
fn test_iec_extremely_inverse_estimation_matches_python_benchmark() {
    let pickup = 1.0;
    // These points come directly from the original curve_estimator.py main() example:
    let points = vec![
        TestPoint::new(2.0, 26.6667),
        TestPoint::new(3.0, 10.0000),
        TestPoint::new(5.0, 3.3333),
    ];

    let report = RelayEstimator::estimate(pickup, &points).expect("Estimation should succeed");
    let best = &report.best_match;

    assert_eq!(best.curve.standard, CurveStandard::IEC);
    assert_eq!(best.curve.id, CurveType::IecExtremelyInverse);
    assert!((best.dial_setting - 1.000).abs() < 1e-3, "TMS should be 1.000, got {:.4}", best.dial_setting);
    assert!(best.rmse < 1e-4, "RMSE should be near 0, got {:.6}", best.rmse);
    assert!(best.fit_quality_score > 99.9, "Fit quality should exceed 99.9%, got {:.2}", best.fit_quality_score);
}

#[test]
fn test_iec_standard_inverse_estimation() {
    let pickup = 1.0;
    let tms = 0.5;
    // Generate synthetic points for IEC Standard Inverse at TMS = 0.5
    // t = 0.14 * 0.5 / ((I/Is)^0.02 - 1)
    let points = vec![
        TestPoint::new(2.0, 0.07 / (2.0f64.powf(0.02) - 1.0)), // ~5.014s
        TestPoint::new(3.0, 0.07 / (3.0f64.powf(0.02) - 1.0)), // ~3.148s
        TestPoint::new(5.0, 0.07 / (5.0f64.powf(0.02) - 1.0)), // ~2.138s
        TestPoint::new(10.0, 0.07 / (10.0f64.powf(0.02) - 1.0)), // ~1.503s
    ];

    let report = RelayEstimator::estimate(pickup, &points).expect("Estimation should succeed");
    let best = &report.best_match;

    assert_eq!(best.curve.standard, CurveStandard::IEC);
    assert!(
        best.curve.id == CurveType::IecStandardInverse || best.curve.id == CurveType::IecNormalInverse,
        "Should identify IEC Standard/Normal Inverse"
    );
    assert!((best.dial_setting - tms).abs() < 1e-3, "TMS should be {:.3}, got {:.4}", tms, best.dial_setting);
    assert!(best.rmse < 1e-4, "RMSE should be near 0");
}

#[test]
fn test_ieee_moderately_inverse_forward_and_inverse() {
    let all = CurveDefinition::all();
    let curve = all
        .iter()
        .find(|c| c.id == CurveType::IeeeModeratelyInverse)
        .expect("IEEE MI curve must exist");

    let pickup = 2.0;
    let td = 1.5;
    let current = 6.0; // 3x pickup

    let trip_time = curve.calculate_operating_time(current, pickup, td).expect("Calculation should succeed");
    assert!(trip_time > 0.0);

    // Test reverse calculation
    let solved_td = curve.calculate_dial_from_point(current, pickup, trip_time).expect("Inverse should succeed");
    assert!((solved_td - td).abs() < 1e-6, "Solved TD should match original TD within epsilon");
}

#[test]
fn test_iec_extremely_inverse_forward_and_inverse() {
    let all = CurveDefinition::all();
    let curve = all
        .iter()
        .find(|c| c.id == CurveType::IecExtremelyInverse)
        .expect("IEC EI curve must exist");

    let pickup = 5.0;
    let tms = 0.4;
    let current = 25.0; // 5x pickup

    let trip_time = curve.calculate_operating_time(current, pickup, tms).expect("Calculation should succeed");
    assert!(trip_time > 0.0);

    // Test reverse calculation
    let solved_tms = curve.calculate_dial_from_point(current, pickup, trip_time).expect("Inverse should succeed");
    assert!((solved_tms - tms).abs() < 1e-6, "Solved TMS should match original TMS");
}

#[test]
fn test_all_ranked_curves_sorted() {
    let pickup = 1.0;
    let points = vec![
        TestPoint::new(2.0, 3.82),
        TestPoint::new(4.0, 1.95),
        TestPoint::new(8.0, 1.10),
    ];

    let report = RelayEstimator::estimate(pickup, &points).expect("Estimation should succeed");
    assert!(!report.all_ranked.is_empty());

    // Check sorted order
    for i in 0..report.all_ranked.len() - 1 {
        assert!(
            report.all_ranked[i].mse <= report.all_ranked[i + 1].mse,
            "Results must be strictly sorted by MSE"
        );
    }
}

#[test]
fn test_invalid_pickup_current() {
    let points = vec![TestPoint::new(2.0, 10.0)];
    let result = RelayEstimator::estimate(0.0, &points);
    assert!(result.is_err(), "Zero pickup current should return error");

    let result_neg = RelayEstimator::estimate(-5.0, &points);
    assert!(result_neg.is_err(), "Negative pickup current should return error");
}

#[test]
fn test_invalid_current_below_pickup() {
    let pickup = 10.0;
    let points = vec![
        TestPoint::new(5.0, 10.0), // Current < Pickup
        TestPoint::new(8.0, 20.0),
    ];

    let result = RelayEstimator::estimate(pickup, &points);
    assert!(result.is_err(), "Points below pickup current should be rejected");
}
