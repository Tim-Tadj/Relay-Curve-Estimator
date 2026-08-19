use crate::curves::CurveDefinition;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestPoint {
    pub current: f64,
    pub time: f64,
    pub active: bool,
    pub label: String,
    #[serde(default)]
    pub current_str: String,
    #[serde(default)]
    pub time_str: String,
}

impl TestPoint {
    pub fn new(current: f64, time: f64) -> Self {
        Self {
            current,
            time,
            active: true,
            label: String::new(),
            current_str: format!("{:.3}", current),
            time_str: format!("{:.4}", time),
        }
    }

    pub fn with_label(current: f64, time: f64, label: impl Into<String>) -> Self {
        Self {
            current,
            time,
            active: true,
            label: label.into(),
            current_str: format!("{:.3}", current),
            time_str: format!("{:.4}", time),
        }
    }

    pub fn sync_strings(&mut self) {
        self.current_str = format!("{:.3}", self.current);
        self.time_str = format!("{:.4}", self.time);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PointVerification {
    pub current: f64,
    pub actual_time: f64,
    pub estimated_time: f64,
    pub abs_error: f64,
    pub rel_error_percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurveFitResult {
    pub curve: CurveDefinition,
    pub dial_setting: f64,
    pub mse: f64,
    pub rmse: f64,
    pub mae: f64,
    pub max_rel_error_percent: f64,
    pub points_evaluated: usize,
    pub verifications: Vec<PointVerification>,
    pub fit_quality_score: f64, // 0.0 to 100.0 score
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EstimationReport {
    pub pickup_current: f64,
    pub best_match: CurveFitResult,
    pub all_ranked: Vec<CurveFitResult>,
    pub total_points_provided: usize,
    pub valid_points_used: usize,
    pub elapsed_micros: u128,
}

pub struct RelayEstimator;

impl RelayEstimator {
    /// Estimates the optimal dial setting for all candidate curves and returns the ranked results
    pub fn estimate(pickup_current: f64, points: &[TestPoint]) -> Result<EstimationReport, String> {
        let start_time = std::time::Instant::now();

        if pickup_current <= 0.0 {
            return Err("Pickup current (Is) must be strictly positive (> 0 A).".to_string());
        }

        // Filter active and valid points where current > pickup_current and time > 0
        let valid_points: Vec<&TestPoint> = points
            .iter()
            .filter(|p| p.active && p.current > pickup_current && p.time > 0.0)
            .collect();

        if valid_points.is_empty() {
            return Err("No valid test points provided. Current must be strictly greater than pickup current (Is) and operating time must be > 0.".to_string());
        }

        let all_curves = CurveDefinition::all();
        let mut results = Vec::with_capacity(all_curves.len());

        for curve in all_curves {
            // Analytical least-squares optimal dial: D = sum(t_i * f_i) / sum(f_i^2) where f_i is base curve time at dial=1.0
            let mut sum_tf = 0.0;
            let mut sum_f2 = 0.0;

            for p in &valid_points {
                if let Some(f_i) = curve.calculate_operating_time(p.current, pickup_current, 1.0) {
                    if f_i > 0.0 {
                        sum_tf += p.time * f_i;
                        sum_f2 += f_i * f_i;
                    }
                }
            }

            let dial_setting = if sum_f2 > 0.0 {
                (sum_tf / sum_f2).max(0.0001)
            } else {
                1.0
            };

            // Compute residuals and verifications
            let mut sum_sq_err = 0.0;
            let mut sum_abs_err = 0.0;
            let mut max_rel_err = 0.0f64;
            let mut verifications = Vec::with_capacity(valid_points.len());

            for p in &valid_points {
                let est_time = curve
                    .calculate_operating_time(p.current, pickup_current, dial_setting)
                    .unwrap_or(0.0);
                let abs_err = (p.time - est_time).abs();
                let sq_err = (p.time - est_time).powi(2);
                let rel_err = if p.time > 0.0 {
                    (abs_err / p.time) * 100.0
                } else {
                    0.0
                };

                sum_sq_err += sq_err;
                sum_abs_err += abs_err;
                if rel_err > max_rel_err {
                    max_rel_err = rel_err;
                }

                verifications.push(PointVerification {
                    current: p.current,
                    actual_time: p.time,
                    estimated_time: est_time,
                    abs_error: abs_err,
                    rel_error_percent: rel_err,
                });
            }

            let n = valid_points.len() as f64;
            let mse = sum_sq_err / n;
            let rmse = mse.sqrt();
            let mae = sum_abs_err / n;

            // Fit Quality Score (0 to 100% where 100% is perfect zero RMSE)
            let avg_time: f64 = valid_points.iter().map(|p| p.time).sum::<f64>() / n;
            let fit_quality_score = if avg_time > 0.0 {
                let normalized_err = rmse / avg_time;
                (100.0 * (-3.0 * normalized_err).exp()).clamp(0.0, 100.0)
            } else {
                0.0
            };

            results.push(CurveFitResult {
                curve,
                dial_setting,
                mse,
                rmse,
                mae,
                max_rel_error_percent: max_rel_err,
                points_evaluated: valid_points.len(),
                verifications,
                fit_quality_score,
            });
        }

        // Sort candidate results by RMSE ascending (best fit first)
        results.sort_by(|a, b| a.rmse.partial_cmp(&b.rmse).unwrap_or(std::cmp::Ordering::Equal));

        let best_match = results
            .first()
            .cloned()
            .ok_or_else(|| "Failed to compute curve fits.".to_string())?;

        let elapsed = start_time.elapsed().as_micros();

        Ok(EstimationReport {
            pickup_current,
            best_match,
            all_ranked: results,
            total_points_provided: points.len(),
            valid_points_used: valid_points.len(),
            elapsed_micros: elapsed,
        })
    }
}
