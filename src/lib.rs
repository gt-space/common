use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt};

/// Encodes possible measurements for every type of sensor on the vehicle.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum Unit {
	/// Preferred unit for pressure readings.
	Psi(f64),

	/// Preferred unit for temperature readings.
	Fahrenheit(f64),

	/// Preferred unit for electric potential readings and raw readings.
	Volts(f64),

	/// No data available.
	NoData,
}

impl fmt::Display for Unit {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Psi(raw) => write!(f, "{raw:.2} psi"),
			Self::Fahrenheit(raw) => write!(f, "{raw:.2} °F"),
			Self::Volts(raw) => write!(f, "{raw} V"),
			Self::NoData => write!(f, "\x1b[31mno data\x1b[0m"),
		}
	}
}

/// Encodes every possible valve state.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum ValveState {
	/// Acknowledged open.
	Open,

	/// Acknowledged closed.
	Closed,

	/// Commanded open, but currently closed.
	CommandedOpen,

	/// Commanded closed, but currently open.
	CommandedClosed,
	
	/// No data on the state of the valve.
	NoData,
}

impl fmt::Display for ValveState {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}", match self {
			Self::Open => "open",
			Self::Closed => "closed",
			Self::CommandedOpen => "commanded open",
			Self::CommandedClosed => "commanded closed",
			Self::NoData => "",
		})
	}
}

impl ValveState {
	/// Converts the valve state into a colored string ready to be displayed on the interface.
	pub fn to_colored_string(&self) -> String {
		match self {
			Self::Open => "\x1b[32mopen\x1b[0m",
			Self::Closed => "\x1b[32mclosed\x1b[0m",
			Self::CommandedOpen => "\x1b[33mclosed\x1b[0m",
			Self::CommandedClosed => "\x1b[33mopen\x1b[0m",
			Self::NoData => "\x1b[31mno data\x1b[0m",
		}.to_owned()
	}
}

/// Holds the state of the vehicle using `HashMap`s which convert a node's name to its state.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct VehicleState {
	/// Holds the current states of all valves on the vehicle.
	pub valve_states: HashMap<String, ValveState>,

	/// Holds the latest readings of all sensors on the vehicle.
	pub sensor_readings: HashMap<String, Unit>,

	/// Holds the last update times for each valve and sensor.
	pub update_times: HashMap<String, f64>,
}

impl VehicleState {
	/// Constructs a new, empty `VehicleState`.
	pub fn new() -> Self {
		VehicleState {
			valve_states: HashMap::new(),
			sensor_readings: HashMap::new(),
			update_times: HashMap::new(),
		}
	}
}
