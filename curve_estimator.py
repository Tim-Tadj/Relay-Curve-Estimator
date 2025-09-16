# import numpy as np
from typing import List, Tuple, Dict
from enum import Enum

def mean(values: List[float]) -> float:
    return sum(values) / len(values)

class CurveStandard(Enum):
    IEC = "IEC"
    IEEE = "IEEE"

class CurveEstimator:
    def __init__(self):
        # IEC curve constants
        self.iec_curves = {
            'SI': {'k': 0.14, 'alpha': 0.02}, # Standard Inverse
            'VI': {'k': 13.5, 'alpha': 1}, # Very Inverse
            'EI': {'k': 80, 'alpha': 2}, # Extremely Inverse
            'NI': {'k': 0.14, 'alpha': 0.02} # Normal Inverse
        }
        
        # IEEE curve constants
        self.ieee_curves = {
            'MI': {'A': 0.0515, 'B': 0.114, 'p': 0.02},   # Moderately Inverse
            'VI': {'A': 19.61, 'B': 0.491, 'p': 2},       # Very Inverse
            'EI': {'A': 28.2, 'B': 0.1217, 'p': 2},       # Extremely Inverse
            'SI': {'A': 0.16758, 'B': 0.11858, 'p': 0.02},# Short-Time Inverse
            'LI': {'A': 0.00262, 'B': 0.00262, 'p': 0.02},# Long-Time Inverse
            'UI': {'A': 8.9341, 'B': 0.17966, 'p': 2}     # Ultra Inverse
        }

    def calculate_operating_time_iec(
        self, curve_type: str, current: float, pickup_current: float, tms: float
    ) -> float:
        """Calculate operating time for IEC curves."""
        k = self.iec_curves[curve_type]['k']
        alpha = self.iec_curves[curve_type]['alpha']
        current_ratio = current / pickup_current
        return (k * tms) / ((current_ratio ** alpha) - 1)

    def calculate_operating_time_ieee(
        self, curve_type: str, current: float, pickup_current: float, td: float
    ) -> float:
        """Calculate operating time for IEEE curves."""
        A = self.ieee_curves[curve_type]['A']
        B = self.ieee_curves[curve_type]['B']
        p = self.ieee_curves[curve_type]['p']
        current_ratio = current / pickup_current
        return td * (A / ((current_ratio ** p) - 1) + B)

    def calculate_tms_iec(
        self,
        curve_type: str,
        current: float,
        pickup_current: float,
        operating_time: float,
    ) -> float:
        """Calculate TMS for IEC curves."""
        k = self.iec_curves[curve_type]['k']
        alpha = self.iec_curves[curve_type]['alpha']
        current_ratio = current / pickup_current
        return operating_time * ((current_ratio ** alpha) - 1) / k

    def calculate_td_ieee(
        self,
        curve_type: str,
        current: float,
        pickup_current: float,
        operating_time: float,
    ) -> float:
        """Calculate Time Dial for IEEE curves."""
        A = self.ieee_curves[curve_type]['A']
        B = self.ieee_curves[curve_type]['B']
        p = self.ieee_curves[curve_type]['p']
        current_ratio = current / pickup_current
        return operating_time / (A / ((current_ratio ** p) - 1) + B)

    def estimate_curve(
        self,
        pickup_current: float,
        test_points: List[Tuple[float, float]],
    ) -> Dict:
        """
        Estimate curve type, standard, and time dial based on test points.
        
        Args:
            pickup_current: Pickup current value
            test_points: List of tuples (current, operating_time)
            
        Returns:
            Dictionary containing best curve parameters
        """
        best_result = {
            'standard': None,
            'curve_type': None,
            'time_dial': 0,
            'error': float('inf')
        }

        # Try IEC curves
        for curve_type in self.iec_curves.keys():
            tms_values = [
                self.calculate_tms_iec(curve_type, current, pickup_current, time)
                for current, time in test_points
            ]
            avg_tms = mean(tms_values)
            
            error = 0
            for current, actual_time in test_points:
                estimated_time = self.calculate_operating_time_iec(
                    curve_type, current, pickup_current, avg_tms
                )
                error += (estimated_time - actual_time) ** 2

            if error < best_result['error']:
                best_result = {
                    'standard': CurveStandard.IEC,
                    'curve_type': curve_type,
                    'time_dial': avg_tms,
                    'error': error
                }

        # Try IEEE curves
        for curve_type in self.ieee_curves.keys():
            td_values = [
                self.calculate_td_ieee(curve_type, current, pickup_current, time)
                for current, time in test_points
            ]
            avg_td = mean(td_values)
            
            error = 0
            for current, actual_time in test_points:
                estimated_time = self.calculate_operating_time_ieee(
                    curve_type, current, pickup_current, avg_td
                )
                error += (estimated_time - actual_time) ** 2

            if error < best_result['error']:
                best_result = {
                    'standard': CurveStandard.IEEE,
                    'curve_type': curve_type,
                    'time_dial': avg_td,
                    'error': error
                }

        return best_result

def main():
    estimator = CurveEstimator()
    
    # Example test case
    pickup_current = 1.0
    test_points = [
        (2.0, 26.6667),  # At 2x pickup current
        (3, 10.0),   # At 3x pickup current
        (5, 3.3333),   # At 5x pickup current
    ]
    
    result = estimator.estimate_curve(pickup_current, test_points)
    
    print(f"Estimated Standard: {result['standard'].value}")
    print(f"Estimated Curve Type: {result['curve_type']}")
    print(f"Estimated Time Dial: {result['time_dial']:.3f}")
    print(f"Mean Square Error: {result['error']:.6f}")
    
    # Verify results
    print("\nVerification:")
    print("Current  Actual Time  Estimated Time")
    print("-" * 40)
    
    for current, actual_time in test_points:
        if result['standard'] == CurveStandard.IEC:
            estimated_time = estimator.calculate_operating_time_iec(
                result['curve_type'],
                current,
                pickup_current,
                result['time_dial']
            )
        else:
            estimated_time = estimator.calculate_operating_time_ieee(
                result['curve_type'],
                current,
                pickup_current,
                result['time_dial']
            )
        
        print(
            f"{current:7.1f}  {actual_time:11.3f}  {estimated_time:13.3f}"
        )

if __name__ == "__main__":
    main()
