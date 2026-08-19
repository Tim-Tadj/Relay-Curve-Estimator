use crate::estimator::TestPoint;

#[derive(Debug, Clone)]
pub struct PresetTestCase {
    pub name: &'static str,
    pub description: &'static str,
    pub pickup_current: f64,
    pub points: Vec<TestPoint>,
}

impl PresetTestCase {
    pub fn all() -> Vec<PresetTestCase> {
        vec![
            PresetTestCase {
                name: "IEC Extremely Inverse (EI) - Transformer Inrush",
                description: "IEC 60255 Extremely Inverse benchmark case (Pickup 1.0A, TMS 1.00).",
                pickup_current: 1.0,
                points: vec![
                    TestPoint::with_label(2.0, 26.6667, "2x Pickup"),
                    TestPoint::with_label(3.0, 10.0000, "3x Pickup"),
                    TestPoint::with_label(5.0, 3.3333, "5x Pickup"),
                ],
            },
            PresetTestCase {
                name: "IEC Standard Inverse (SI) - Feeder Relay",
                description: "IEC 60255 Standard Inverse general distribution feeder overcurrent (Pickup 1.0A, TMS 0.50).",
                pickup_current: 1.0,
                points: vec![
                    TestPoint::with_label(2.0, 5.0145, "2x Pickup"),
                    TestPoint::with_label(3.0, 3.1477, "3x Pickup"),
                    TestPoint::with_label(5.0, 2.1382, "5x Pickup"),
                    TestPoint::with_label(10.0, 1.5032, "10x Pickup"),
                ],
            },
            PresetTestCase {
                name: "IEC Very Inverse (VI) - Feeder Fault",
                description: "IEC 60255 Very Inverse curve with TMS = 0.50 (Pickup 5.0A).",
                pickup_current: 5.0,
                points: vec![
                    TestPoint::with_label(10.0, 6.750, "2x Pickup"),
                    TestPoint::with_label(15.0, 3.375, "3x Pickup"),
                    TestPoint::with_label(25.0, 1.688, "5x Pickup"),
                    TestPoint::with_label(50.0, 0.750, "10x Pickup"),
                ],
            },
            PresetTestCase {
                name: "IEEE Moderately Inverse (MI) - Distribution",
                description: "IEEE C37.112 Moderately Inverse characteristic with TD = 1.0 (Pickup 1.0A).",
                pickup_current: 1.0,
                points: vec![
                    TestPoint::with_label(2.0, 3.821, "2x Pickup"),
                    TestPoint::with_label(3.0, 2.454, "3x Pickup"),
                    TestPoint::with_label(5.0, 1.506, "5x Pickup"),
                    TestPoint::with_label(10.0, 0.824, "10x Pickup"),
                ],
            },
            PresetTestCase {
                name: "IEEE Very Inverse (VI) - Heavy Overcurrent",
                description: "IEEE C37.112 Very Inverse characteristic with TD = 2.0 (Pickup 5.0A).",
                pickup_current: 5.0,
                points: vec![
                    TestPoint::with_label(10.0, 14.055, "2x Pickup"),
                    TestPoint::with_label(15.0, 5.884, "3x Pickup"),
                    TestPoint::with_label(25.0, 2.618, "5x Pickup"),
                    TestPoint::with_label(50.0, 1.380, "10x Pickup"),
                ],
            },
            PresetTestCase {
                name: "Motor Thermal Overload (IEC Long-Time Inverse)",
                description: "IEC Long-Time Inverse overload curve for motor thermal backup (Pickup 10.0A, TMS 0.10).",
                pickup_current: 10.0,
                points: vec![
                    TestPoint::with_label(15.0, 24.00, "1.5x Pickup"),
                    TestPoint::with_label(20.0, 12.00, "2.0x Pickup"),
                    TestPoint::with_label(30.0, 6.00, "3.0x Pickup"),
                    TestPoint::with_label(50.0, 3.00, "5.0x Pickup"),
                ],
            },
        ]
    }
}
