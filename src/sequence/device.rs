use jeflog::{fail, warn};
use pyo3::{pyclass, pyclass::CompareOp, pymethods, types::PyNone, PyAny, IntoPy, PyObject, PyResult, Python, ToPyObject};
use std::net::{SocketAddr, ToSocketAddrs, UdpSocket};
use crate::comm::SamControlMessage;

use super::{MAPPINGS, SAM_SOCKET, VEHICLE_STATE};

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

		if let Some(vehicle_state) = VEHICLE_STATE.get() {
			if let Ok(vehicle_state) = vehicle_state.lock() {
				measurement = vehicle_state.sensor_readings
					.get(&self.name)
					.cloned();
			} else {
				fail!("Failed to lock vehicle state in method common::sequence::Sensor::read.");
			}
		} else {
			fail!("Sequences library has not been initialized. Call the initialize function before running a sequence.");
		}

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
		let Some(mappings) = MAPPINGS.get() else {
			fail!("Sequences library has not been initialized. Call the initialize function before running a sequence.");
			return;
		};

		let Ok(mappings) = mappings.lock() else {
			fail!("Failed to lock mappings in method common::sequence::Valve::actuate.");
			return;
		};

		// TODO: replace with tcp socket
		let socket = SAM_SOCKET.get_or_init(|| UdpSocket::bind(("0.0.0.0", 0)).unwrap());

		let Some(mapping) = mappings.iter().find(|mapping| mapping.text_id == self.name) else {
			fail!("Failed to actuate valve: mapping {} is not defined.", self.name);
			return;
		};

		let normally_closed = match mapping.normally_closed {
			Some(nc) => nc,
			None => {
				warn!("Normal state not defined for {}. Default to normally closed.", mapping.text_id);
				true
			}
		};
		
		let message = SamControlMessage::ActuateValve {
			channel: mapping.channel,
			powered: normally_closed == open
		};

		// TODO: switch this soon
		let address = format!("{}.local:8378", mapping.board_id)
			.to_socket_addrs()
			.ok()
			.and_then(|mut addresses| addresses.find(SocketAddr::is_ipv4));

		if let Some(address) = address {
			let Ok(serialized) = postcard::to_allocvec(&message) else {
				fail!("Failed to serialize valve actuation control message.");
				return;
			};

			if let Err(error) = socket.send_to(&serialized, address) {
				fail!("Failed to send valve actuation command: {error}");
				return;
			}
		} else {
			fail!("Address of board '{}' could not be located.", mapping.board_id);
			return;
		}
	}
}
