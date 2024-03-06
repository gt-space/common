use crate::comm::ValveState;
use jeflog::fail;
use pyo3::{pyclass, pyclass::CompareOp, pymethods, types::PyNone, IntoPy, PyAny, PyObject, PyResult, Python, ToPyObject};
use super::{DeviceAction, DEVICE_HANDLER};

/// A Python-exposed class that allows for interacting with a sensor.
#[pyclass]
#[derive(Clone, Debug)]
pub struct Sensor {
	name: String
}

#[pymethods]
impl Sensor {
	/// Creates a new sensor with the specified text identifier.
	#[new]
	pub fn new(name: String) -> Self {
		Sensor { name }
	}

	/// Reads the latest sensor measurements by indexing into the global vehicle state.
	pub fn read(&self) -> PyObject {
		let Some(device_handler) = &*DEVICE_HANDLER.lock().unwrap() else {
			fail!("Device handler not set before accessing external device.");
			return Python::with_gil(|py| PyNone::get(py).to_object(py));
		};

		let measurement = device_handler(&self.name, DeviceAction::ReadSensor);

		Python::with_gil(|py| {
			measurement.map_or(
				PyNone::get(py).to_object(py),
				|measurement| measurement.into_py(py),
			)
		})
	}

	fn __richcmp__(&self, other: &PyAny, op: CompareOp) -> PyResult<bool> {
		Ok(other.rich_compare(self.read(), op)?.is_true()?)
	}
}

/// A Python-exposed class that allows for interacting with a valve.
#[pyclass]
#[derive(Clone, Debug)]
pub struct Valve {
	name: String
}

#[pymethods]
impl Valve {
	/// Constructs a new `Valve` with its mapping's text ID.
	#[new]
	pub fn new(name: String) -> Self {
		Valve { name }
	}

	/// Instructs the SAM board to open the valve.
	pub fn open(&self) {
		self.actuate(true);
	}

	/// Instructs the SAM board to close the valve.
	pub fn close(&self) {
		self.actuate(false);
	}

	/// Instructs the SAM board to actuate a valve.
	pub fn actuate(&self, open: bool) {
		let Some(device_handler) = &*DEVICE_HANDLER.lock().unwrap() else {
			fail!("Device handler not set before accessing external device.");
			return;
		};

		let state = if open { ValveState::Open } else { ValveState::Closed };
		device_handler(&self.name, DeviceAction::ActuateValve { state });
	}
}
