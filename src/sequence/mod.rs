mod device;
mod func;
mod unit;

pub use device::*;
pub use func::*;
use jeflog::{fail, warn};
use pyo3::{pymodule, types::PyModule, wrap_pyfunction, Py, PyResult, Python};
pub use unit::*;

use crate::comm::{NodeMapping, Sequence, VehicleState, ChannelType};
use std::{net::UdpSocket, sync::{Arc, Mutex, OnceLock}};

#[pymodule]
fn sequences(py: Python<'_>, module: &PyModule) -> PyResult<()> {
	module.add_class::<Current>()?;
	module.add_class::<Duration>()?;
	module.add_class::<ElectricPotential>()?;
	module.add_class::<Force>()?;
	module.add_class::<Pressure>()?;
	module.add_class::<Temperature>()?;

	module.add("A", Py::new(py, Current::new(1.0))?)?;
	module.add("mA", Py::new(py, Current::new(0.001))?)?;
	module.add("s", Py::new(py, Duration::new(1.0))?)?;
	module.add("ms", Py::new(py, Duration::new(0.001))?)?;
	module.add("us", Py::new(py, Duration::new(0.000001))?)?;
	module.add("V", Py::new(py, ElectricPotential::new(1.0))?)?;
	module.add("mV", Py::new(py, ElectricPotential::new(0.001))?)?;
	module.add("lbf", Py::new(py, Force::new(1.0))?)?;
	module.add("psi", Py::new(py, Pressure::new(1.0))?)?;
	module.add("K", Py::new(py, Temperature::new(1.0))?)?;

	module.add_class::<Sensor>()?;
	module.add_class::<Valve>()?;

	module.add_function(wrap_pyfunction!(wait_for, module)?)?;
	module.add_function(wrap_pyfunction!(wait_until, module)?)?;

	Ok(())
}

// these are global, static variables defined for use in the sequences library and nowhere else.
// they are necessary for passing this global state to be read or written to in the Valve and Sensor methods.
// all of these should be properly and fully defined with initialize.
pub(crate) static VEHICLE_STATE: OnceLock<Arc<Mutex<VehicleState>>> = OnceLock::new();
pub(crate) static SAM_SOCKET: OnceLock<UdpSocket> = OnceLock::new();
pub(crate) static MAPPINGS: OnceLock<Arc<Mutex<Vec<NodeMapping>>>> = OnceLock::new();

/// Initializes the sequences portion of the library.
pub fn initialize(vehicle_state: Arc<Mutex<VehicleState>>, mappings: Arc<Mutex<Vec<NodeMapping>>>) {
	if VEHICLE_STATE.set(vehicle_state).is_err() || MAPPINGS.set(mappings).is_err() {
		warn!("Sequences library has already been initialized. Ignoring reinitialization.");
		return;
	}

	pyo3::append_to_inittab!(sequences);
	pyo3::prepare_freethreaded_python();
}

// TODO: change the run function to return an error in the event of one instead of printing out the error

/// Runs a sequence. The `initialize` function must be called before this.
pub fn run(sequence: Sequence) {
	let Some(mappings) = MAPPINGS.get() else {
		fail!("Sequences library has not been initialized. Call the initialize function before running a sequence.");
		return;
	};

	let Ok(mappings) = mappings.lock() else {
		fail!("Mappings could not be locked within common::sequence::run.");
		return;
	};

	Python::with_gil(|py| {
		if let Err(error) = py.run("from sequences import *", None, None) {
			fail!("Failed to import sequences library: {error}");
			return;
		}

		for mapping in &*mappings {
			// TODO: inspect this definition again. this may be redefining values unnecessarily.
			let definition = match mapping.channel_type {
				ChannelType::ValveCurrent => format!("{0} = Valve('{0}')", mapping.text_id),
				_ => format!("{0} = Sensor('{0}')", mapping.text_id),
			};

			if let Err(error) = py.run(&definition, None, None) {
				fail!("Failed to define {} as a mapping: {error}", mapping.text_id);
				return;
			}
		}

		// drop the lock before entering script to prevent deadlock
		drop(mappings);

		if let Err(error) = py.run(&sequence.script, None, None) {
			fail!("Failed to run sequence '{}': {error}", sequence.name);
		}
	});
}
