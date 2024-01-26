use std::net::UdpSocket;

use pyo3::{pyclass, pyclass::CompareOp, pymethods, types::PyNone, IntoPy, PyAny, PyObject, PyResult, Python, ToPyObject};
use super::{SAM_SOCKET, VEHICLE_STATE};

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
		let mut measurement = None;
		let vehicle_state = VEHICLE_STATE.get();

		if let Some(vehicle_state) = vehicle_state {
			if let Ok(vehicle_state) = vehicle_state.lock() {
				measurement = vehicle_state.sensor_readings
					.get(&self.name)
					.cloned();
			}
		}

		Python::with_gil(|py| {
			measurement.map_or(
				PyNone::get(py).to_object(py),
				|measurement| measurement.into_py(py)
			)
		})
	}

	fn __richcmp__(&self, other: &PyAny, op: CompareOp) -> PyResult<bool> {
		Ok(other.rich_compare(self.read(), op)?.is_true()?)
	}
}

#[pyclass]
#[derive(Clone, Debug)]
pub struct Valve {
	name: String
}

#[pymethods]
impl Valve {
	#[new]
	pub fn new(name: String) -> Self {
		Valve { name }
	}

	/// Instructs the SAM board to open the valve.
	pub fn open(&self) {
		let socket = SAM_SOCKET.get_or_init(|| UdpSocket::bind(("0.0.0.0", 0)).unwrap());
		
	}

	/// Instructs the SAM board to close the valve.
	pub fn close(&self) {

	}
}
