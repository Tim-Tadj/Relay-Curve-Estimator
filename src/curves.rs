use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CurveStandard {
    IEC,
    IEEE,
}

impl CurveStandard {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::IEC => "IEC 60255",
            Self::IEEE => "IEEE C37.112",
        }
    }

    pub fn dial_name(&self) -> &'static str {
        match self {
            Self::IEC => "TMS (Time Multiplier)",
            Self::IEEE => "TD (Time Dial)",
        }
    }

    pub fn dial_short_name(&self) -> &'static str {
        match self {
            Self::IEC => "TMS",
            Self::IEEE => "TD",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IecParams {
    pub k: f64,
    pub alpha: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IeeeParams {
    pub a: f64,
    pub b: f64,
    pub p: f64,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CurveType {
    // IEC Curves
    IecStandardInverse,
    IecVeryInverse,
    IecExtremelyInverse,
    IecLongTimeInverse,
    IecNormalInverse,

    // IEEE Curves
    IeeeModeratelyInverse,
    IeeeVeryInverse,
    IeeeExtremelyInverse,
    IeeeShortTimeInverse,
    IeeeLongTimeInverse,
    IeeeUltraInverse,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurveDefinition {
    pub id: CurveType,
    pub standard: CurveStandard,
    pub code: String,
    pub name: String,
    pub description: String,
    pub formula_str: String,
}

impl CurveDefinition {
    pub fn all() -> Vec<CurveDefinition> {
        vec![
            // IEC
            CurveDefinition {
                id: CurveType::IecStandardInverse,
                standard: CurveStandard::IEC,
                code: "SI".to_string(),
                name: "Standard Inverse (SI)".to_string(),
                description: "IEC 60255 Standard Inverse characteristic. Most commonly used for general feeder protection.".to_string(),
                formula_str: "t = (0.14 × TMS) / ((I/Is)^0.02 - 1)".to_string(),
            },
            CurveDefinition {
                id: CurveType::IecVeryInverse,
                standard: CurveStandard::IEC,
                code: "VI".to_string(),
                name: "Very Inverse (VI)".to_string(),
                description: "IEC 60255 Very Inverse. Steeper curve ideal where fault current drops significantly with distance.".to_string(),
                formula_str: "t = (13.5 × TMS) / ((I/Is)^1.0 - 1)".to_string(),
            },
            CurveDefinition {
                id: CurveType::IecExtremelyInverse,
                standard: CurveStandard::IEC,
                code: "EI".to_string(),
                name: "Extremely Inverse (EI)".to_string(),
                description: "IEC 60255 Extremely Inverse. Highly steep curve suitable for transformer inrush and fuse coordination.".to_string(),
                formula_str: "t = (80.0 × TMS) / ((I/Is)^2.0 - 1)".to_string(),
            },
            CurveDefinition {
                id: CurveType::IecLongTimeInverse,
                standard: CurveStandard::IEC,
                code: "LTI".to_string(),
                name: "Long Time Inverse (LTI)".to_string(),
                description: "IEC 60255 Long Time Inverse for motor overload and thermal backup protection.".to_string(),
                formula_str: "t = (120.0 × TMS) / ((I/Is)^1.0 - 1)".to_string(),
            },
            CurveDefinition {
                id: CurveType::IecNormalInverse,
                standard: CurveStandard::IEC,
                code: "NI".to_string(),
                name: "Normal Inverse (NI)".to_string(),
                description: "IEC 60255 Normal Inverse (equivalent constant parameters to Standard Inverse).".to_string(),
                formula_str: "t = (0.14 × TMS) / ((I/Is)^0.02 - 1)".to_string(),
            },

            // IEEE
            CurveDefinition {
                id: CurveType::IeeeModeratelyInverse,
                standard: CurveStandard::IEEE,
                code: "MI".to_string(),
                name: "Moderately Inverse (MI)".to_string(),
                description: "IEEE C37.112 Moderately Inverse characteristic. Common general distribution overcurrent protection.".to_string(),
                formula_str: "t = TD × [ 0.0515 / ((I/Is)^0.02 - 1) + 0.114 ]".to_string(),
            },
            CurveDefinition {
                id: CurveType::IeeeVeryInverse,
                standard: CurveStandard::IEEE,
                code: "VI".to_string(),
                name: "Very Inverse (VI)".to_string(),
                description: "IEEE C37.112 Very Inverse characteristic with quadratic exponent.".to_string(),
                formula_str: "t = TD × [ 19.61 / ((I/Is)^2.0 - 1) + 0.491 ]".to_string(),
            },
            CurveDefinition {
                id: CurveType::IeeeExtremelyInverse,
                standard: CurveStandard::IEEE,
                code: "EI".to_string(),
                name: "Extremely Inverse (EI)".to_string(),
                description: "IEEE C37.112 Extremely Inverse characteristic for fast clearing at high fault levels.".to_string(),
                formula_str: "t = TD × [ 28.2 / ((I/Is)^2.0 - 1) + 0.1217 ]".to_string(),
            },
            CurveDefinition {
                id: CurveType::IeeeShortTimeInverse,
                standard: CurveStandard::IEEE,
                code: "SI".to_string(),
                name: "Short-Time Inverse (SI)".to_string(),
                description: "IEEE C37.112 Short-Time Inverse. Fast response curve for selective coordination.".to_string(),
                formula_str: "t = TD × [ 0.16758 / ((I/Is)^0.02 - 1) + 0.11858 ]".to_string(),
            },
            CurveDefinition {
                id: CurveType::IeeeLongTimeInverse,
                standard: CurveStandard::IEEE,
                code: "LI".to_string(),
                name: "Long-Time Inverse (LI)".to_string(),
                description: "IEEE C37.112 Long-Time Inverse for equipment thermal and overload protection.".to_string(),
                formula_str: "t = TD × [ 0.00262 / ((I/Is)^0.02 - 1) + 0.00262 ]".to_string(),
            },
            CurveDefinition {
                id: CurveType::IeeeUltraInverse,
                standard: CurveStandard::IEEE,
                code: "UI".to_string(),
                name: "Ultra Inverse (UI)".to_string(),
                description: "IEEE C37.112 Ultra Inverse characteristic with fast tripping under heavy faults.".to_string(),
                formula_str: "t = TD × [ 8.9341 / ((I/Is)^2.0 - 1) + 0.17966 ]".to_string(),
            },
        ]
    }

    pub fn get_iec_params(&self) -> Option<IecParams> {
        match self.id {
            CurveType::IecStandardInverse | CurveType::IecNormalInverse => Some(IecParams { k: 0.14, alpha: 0.02 }),
            CurveType::IecVeryInverse => Some(IecParams { k: 13.5, alpha: 1.0 }),
            CurveType::IecExtremelyInverse => Some(IecParams { k: 80.0, alpha: 2.0 }),
            CurveType::IecLongTimeInverse => Some(IecParams { k: 120.0, alpha: 1.0 }),
            _ => None,
        }
    }

    pub fn get_ieee_params(&self) -> Option<IeeeParams> {
        match self.id {
            CurveType::IeeeModeratelyInverse => Some(IeeeParams { a: 0.0515, b: 0.114, p: 0.02 }),
            CurveType::IeeeVeryInverse => Some(IeeeParams { a: 19.61, b: 0.491, p: 2.0 }),
            CurveType::IeeeExtremelyInverse => Some(IeeeParams { a: 28.2, b: 0.1217, p: 2.0 }),
            CurveType::IeeeShortTimeInverse => Some(IeeeParams { a: 0.16758, b: 0.11858, p: 0.02 }),
            CurveType::IeeeLongTimeInverse => Some(IeeeParams { a: 0.00262, b: 0.00262, p: 0.02 }),
            CurveType::IeeeUltraInverse => Some(IeeeParams { a: 8.9341, b: 0.17966, p: 2.0 }),
            _ => None,
        }
    }

    /// Calculates forward operating time (seconds) for a given current, pickup current, and dial setting (TMS or TD).
    pub fn calculate_operating_time(&self, current: f64, pickup_current: f64, dial_setting: f64) -> Option<f64> {
        if pickup_current <= 0.0 || current <= pickup_current || dial_setting <= 0.0 {
            return None;
        }

        let m = current / pickup_current; // Current multiple

        match self.standard {
            CurveStandard::IEC => {
                let params = self.get_iec_params()?;
                let denom = m.powf(params.alpha) - 1.0;
                if denom <= 0.0 {
                    return None;
                }
                Some((params.k * dial_setting) / denom)
            }
            CurveStandard::IEEE => {
                let params = self.get_ieee_params()?;
                let denom = m.powf(params.p) - 1.0;
                if denom <= 0.0 {
                    return None;
                }
                Some(dial_setting * (params.a / denom + params.b))
            }
        }
    }

    /// Solves for the required dial setting (TMS or TD) to trip at `operating_time` for a given `current`.
    pub fn calculate_dial_from_point(&self, current: f64, pickup_current: f64, operating_time: f64) -> Option<f64> {
        if pickup_current <= 0.0 || current <= pickup_current || operating_time <= 0.0 {
            return None;
        }

        let m = current / pickup_current;

        match self.standard {
            CurveStandard::IEC => {
                let params = self.get_iec_params()?;
                let denom = m.powf(params.alpha) - 1.0;
                if denom <= 0.0 {
                    return None;
                }
                // t = (k * TMS) / denom  =>  TMS = (t * denom) / k
                Some((operating_time * denom) / params.k)
            }
            CurveStandard::IEEE => {
                let params = self.get_ieee_params()?;
                let denom = m.powf(params.p) - 1.0;
                if denom <= 0.0 {
                    return None;
                }
                let bracket = params.a / denom + params.b;
                if bracket <= 0.0 {
                    return None;
                }
                // t = TD * bracket  =>  TD = t / bracket
                Some(operating_time / bracket)
            }
        }
    }
}
