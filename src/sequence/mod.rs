pub mod device;
pub mod func;
pub mod unit;

pub use device::*;
pub use func::*;
use jeflog::warn;
use pyo3::{pymodule, types::PyModule, wrap_pyfunction, Py, PyResult, Python};
pub use unit::*;

use crate::comm::{Sequence, VehicleState};
use std::{net::UdpSocket, sync::{Arc, Mutex, OnceLock}};

#[pymodule]
pub fn sequences(py: Python<'_>, module: &PyModule) -> PyResult<()> {
	module.add_class::<Duration>()?;
	module.add_class::<Pressure>()?;
	module.add_class::<ElectricPotential>()?;
	module.add_class::<Temperature>()?;

	module.add("s", Py::new(py, Duration::new(1.0))?)?;
	module.add("ms", Py::new(py, Duration::new(0.001))?)?;
	module.add("us", Py::new(py, Duration::new(0.000001))?)?;
	module.add("psi", Py::new(py, Pressure::new(1.0))?)?;
	module.add("V", Py::new(py, ElectricPotential::new(1.0))?)?;
	module.add("F", Py::new(py, Temperature::new(1.0))?)?;

	module.add_class::<Sensor>()?;
	module.add_class::<Valve>()?;

	module.add_function(wrap_pyfunction!(wait_for, module)?)?;
	module.add_function(wrap_pyfunction!(wait_until, module)?)?;

	Ok(())
}

pub(crate) static VEHICLE_STATE: OnceLock<Arc<Mutex<VehicleState>>> = OnceLock::new();
pub(crate) static SAM_SOCKET: OnceLock<UdpSocket> = OnceLock::new();

/// Initializes the sequences portion of the library.
pub fn initialize(vehicle_state: Arc<Mutex<VehicleState>>) {
	if VEHICLE_STATE.set(vehicle_state).is_err() {
		warn!("Sequences library has already been initialized. Ignoring reinitialization.");
		return;
	}

	pyo3::prepare_freethreaded_python();
}

/// Runs a sequence. The `initialize` function must be called before this.
pub fn run(sequence: Sequence) {

}
