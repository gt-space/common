use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt};

pub trait ToPrettyString {
	fn to_pretty_string(&self) -> String;
}

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

// TODO: Decide on the difference between fmt::Display and ToString
// and decide which one to do for what
impl fmt::Display for Unit {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Psi(raw) => write!(f, "{raw} psi"),
			Self::Fahrenheit(raw) => write!(f, "{raw} °F"),
			Self::Volts(raw) => write!(f, "{raw} V"),
			Self::NoData => write!(f, ""),
		}
	}
}

impl ToPrettyString for Unit {
	fn to_pretty_string(&self) -> String {
		match self {
			Self::Psi(raw) => format!("{raw:.2} psi"),
			Self::Fahrenheit(raw) => format!("{raw:.2} °F"),
			Self::Volts(raw) => format!("{raw:.2} V"),
			Self::NoData => "\x1b[31mno data\x1b[0m".to_owned(),
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

impl ToPrettyString for ValveState {
	/// Converts the valve state into a colored string ready to be displayed on the interface.
	fn to_pretty_string(&self) -> String {
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

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ChannelType {
	CurrentLoop,
	ValveVoltage,
	ValveCurrent,
	RailVoltage,
	RailCurrent,
	DifferentialSignal,
	Rtd,
	Tc,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Computer {
	Flight,
	Ground,
}

/// The mapping of an individual node.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NodeMapping {
	/// The text identifier, or name, of the node.
	pub text_id: String,
	
	/// A number identifying which SAM board the node is on.
	pub board_id: u32,

	/// The channel type of the node, such as "valve".
	pub channel_type: ChannelType,

	/// A number identifying which channel on the SAM board controls the node.
	pub channel: u32,

	/// Which computer controls the SAM board, "flight" or "ground".
	pub computer: Computer,
}
