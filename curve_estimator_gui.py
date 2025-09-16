import sys
from PySide6.QtWidgets import (
    QApplication,
    QMainWindow,
    QWidget,
    QVBoxLayout,
    QHBoxLayout,
    QLabel,
    QLineEdit,
    QPushButton,
    QGroupBox,
    QGridLayout,
    QTableWidget,
    QTableWidgetItem,
)
from PySide6.QtCore import Qt
from curve_estimator import CurveEstimator, CurveStandard

class CurveEstimatorGUI(QMainWindow):
    def __init__(self):
        super().__init__()
        self.setWindowTitle("Protection Curve Estimator")
        self.setMinimumWidth(600)
        
        # Create main widget and layout
        main_widget = QWidget()
        self.setCentralWidget(main_widget)
        main_layout = QVBoxLayout(main_widget)
        
        # Create input groups
        self.create_pickup_group()
        self.create_test_points_group()
        self.create_results_group()
        
        # Add groups to main layout
        main_layout.addWidget(self.pickup_group)
        main_layout.addWidget(self.test_points_group)
        main_layout.addWidget(self.results_group)
        
        # Add estimate button
        self.estimate_button = QPushButton("Estimate Curve")
        self.estimate_button.clicked.connect(self.estimate_curve)
        main_layout.addWidget(self.estimate_button)
        
        # Initialize curve estimator
        self.estimator = CurveEstimator()

    def create_pickup_group(self):
        self.pickup_group = QGroupBox("Pickup Current")
        layout = QHBoxLayout()
        
        self.pickup_input = QLineEdit()
        self.pickup_input.setPlaceholderText("Enter pickup current")
        
        layout.addWidget(QLabel("Pickup Current:"))
        layout.addWidget(self.pickup_input)
        self.pickup_group.setLayout(layout)

    def create_test_points_group(self):
        self.test_points_group = QGroupBox("Test Points")
        layout = QGridLayout()
        
        # Headers
        layout.addWidget(QLabel("Point"), 0, 0)
        layout.addWidget(QLabel("Current (A)"), 0, 1)
        layout.addWidget(QLabel("Time (s)"), 0, 2)
        
        # Create input fields for test points
        self.current_inputs = []
        self.time_inputs = []
        
        for i in range(3):
            layout.addWidget(QLabel(f"Point {i+1}:"), i+1, 0)
            
            current_input = QLineEdit()
            current_input.setPlaceholderText(f"Current {i+1}")
            layout.addWidget(current_input, i+1, 1)
            self.current_inputs.append(current_input)
            
            time_input = QLineEdit()
            time_input.setPlaceholderText(f"Time {i+1}")
            layout.addWidget(time_input, i+1, 2)
            self.time_inputs.append(time_input)
        
        self.test_points_group.setLayout(layout)

    def create_results_group(self):
        self.results_group = QGroupBox("Results")
        layout = QVBoxLayout()
        
        # Results labels
        results_grid = QGridLayout()
        self.standard_label = QLabel("Standard: ")
        self.curve_type_label = QLabel("Curve Type: ")
        self.time_dial_label = QLabel("Time Dial: ")
        self.error_label = QLabel("Mean Square Error: ")
        
        results_grid.addWidget(QLabel("Standard:"), 0, 0)
        results_grid.addWidget(self.standard_label, 0, 1)
        results_grid.addWidget(QLabel("Curve Type:"), 0, 2)
        results_grid.addWidget(self.curve_type_label, 0, 3)
        results_grid.addWidget(QLabel("Time Dial:"), 1, 0)
        results_grid.addWidget(self.time_dial_label, 1, 1)
        results_grid.addWidget(QLabel("Mean Square Error:"), 1, 2)
        results_grid.addWidget(self.error_label, 1, 3)
        
        # Verification table
        self.verification_table = QTableWidget(3, 3)
        self.verification_table.setHorizontalHeaderLabels(
            ["Current", "Actual Time", "Estimated Time"]
        )
        self.verification_table.horizontalHeader().setStretchLastSection(True)
        
        layout.addLayout(results_grid)
        layout.addWidget(QLabel("Verification:"))
        layout.addWidget(self.verification_table)
        
        self.results_group.setLayout(layout)

    def get_float_value(self, line_edit, default=0.0):
        try:
            return float(line_edit.text())
        except ValueError:
            return default

    def estimate_curve(self):
        try:
            # Get pickup current
            pickup_current = self.get_float_value(self.pickup_input)
            if pickup_current <= 0:
                raise ValueError("Pickup current must be positive")
            
            # Get test points
            test_points = []
            for current_input, time_input in zip(self.current_inputs, self.time_inputs):
                current = self.get_float_value(current_input)
                time = self.get_float_value(time_input)
                
                if current <= 0 or time <= 0:
                    raise ValueError("Current and time values must be positive")
                
                test_points.append((current, time))
            
            # Perform estimation
            result = self.estimator.estimate_curve(pickup_current, test_points)
            
            # Update results labels
            self.standard_label.setText(result['standard'].value)
            self.curve_type_label.setText(result['curve_type'])
            self.time_dial_label.setText(f"{result['time_dial']:.3f}")
            self.error_label.setText(f"{result['error']:.6f}")
            
            # Update verification table
            for i, (current, actual_time) in enumerate(test_points):
                if result['standard'] == CurveStandard.IEC:
                    estimated_time = self.estimator.calculate_operating_time_iec(
                        result['curve_type'],
                        current,
                        pickup_current,
                        result['time_dial']
                    )
                else:
                    estimated_time = self.estimator.calculate_operating_time_ieee(
                        result['curve_type'],
                        current,
                        pickup_current,
                        result['time_dial']
                    )
                
                self.verification_table.setItem(
                    i, 0, QTableWidgetItem(f"{current:.3f}")
                )
                self.verification_table.setItem(
                    i, 1, QTableWidgetItem(f"{actual_time:.3f}")
                )
                self.verification_table.setItem(
                    i, 2, QTableWidgetItem(f"{estimated_time:.3f}")
                )
            
        except ValueError as e:
            self.standard_label.setText("Error")
            self.curve_type_label.setText(str(e))
            self.time_dial_label.setText("")
            self.error_label.setText("")
            
        except Exception as e:
            self.standard_label.setText("Error")
            self.curve_type_label.setText(str(e))
            self.time_dial_label.setText("")
            self.error_label.setText("")

def main():
    app = QApplication(sys.argv)
    window = CurveEstimatorGUI()
    window.show()
    sys.exit(app.exec())

if __name__ == "__main__":
    main()
