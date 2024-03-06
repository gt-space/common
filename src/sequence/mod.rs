mod device;
mod func;
mod unit;

pub use device::*;
pub use func::*;
use jeflog::{fail, warn};
use pyo3::{pymodule, types::PyModule, wrap_pyfunction, Py, PyResult, Python};
pub use unit::*;

use crate::comm::{ChannelType, Measurement, NodeMapping, Sequence, ValveState, VehicleState};
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

// let's break this one down:
// Mutex<...> - required because this is a global variable, so needed to implement Sync and be used across threads safely
// Option<...> - before initialization by set_device_handler, this will be None, so necessary for compiler to be happy
// Box<dyn ...> - wraps the enclosed dynamic type on the heap, because it's exact size and type are unknown at compile-time
// Fn(&str, DeviceAction) -> Option<Measurement> - the trait bound of the type of the closure being stored, with its arguments and return value
// + Send - requires that everything captured in the closure be safe to send across threads
pub(crate) static DEVICE_HANDLER: Mutex<Option<Box<dyn Fn(&str, DeviceAction) -> Option<Measurement> + Send>>> = Mutex::new(None);
pub(crate) static MAPPINGS: OnceLock<Arc<Mutex<Vec<NodeMapping>>>> = OnceLock::new();
pub(crate) static SAM_SOCKET: OnceLock<UdpSocket> = OnceLock::new();

/// Initializes the sequences portion of the library.
pub fn initialize(mappings: Arc<Mutex<Vec<NodeMapping>>>) {
	if MAPPINGS.set(mappings).is_err() {
		warn!("Sequences library has already been initialized. Ignoring reinitialization.");
		return;
	}

	pyo3::append_to_inittab!(sequences);
	pyo3::prepare_freethreaded_python();
}

/// Given to the device handler to instruct it to perform a type of action.
pub enum DeviceAction {
	/// Instructs to read and return a sensor value.
	ReadSensor,

	/// Instructs to command a valve actuation to match the given state.
	ActuateValve {
		/// The state which the valve should be actuated to match, either `Open` or `Closed`.
		state: ValveState
	},
}

/// Sets the device handler callback, which interacts with external boards from the flight computer code.
/// 
/// The first argument of this callback is a `&str` which is the name of the target device (typically a valve or sensor),
/// and the second argument is the action to be performed by the handler. The return value is an `Option<Measurement>` because in
/// the event of a read, a measurement will need to be returned, but a valve actuation requires no return.
pub fn set_device_handler(handler: impl Fn(&str, DeviceAction) -> Option<Measurement> + Send + 'static) {
	let Ok(mut device_handler) = DEVICE_HANDLER.lock() else {
		fail!("Failed to lock global device handler: Mutex is poisoned.");
		return;
	};

	*device_handler = Some(Box::new(handler));
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
