#![warn(missing_docs)]

//! Common consists of the common shared types between different parts of the YJSP software stack.
//! More specifically, the types sent across the network between the flight computer, control server,
//! GUI, and SAM boards are all stored here.

use rusqlite::{ToSql, types::{ToSqlOutput, ValueRef, FromSql, FromSqlResult, FromSqlError}};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt};

/// Trait providing a method to create a pretty, terminal-friendly representation of the underlying.
pub trait ToPrettyString {
	/// Provides a representation of the underlying which is preferable when displaying to the console but
	/// not as a raw string. ANSI codes such as color codes, for example, can be used in a "pretty string"
	/// but would be atypical in a `fmt::Display` implementation.
	fn to_pretty_string(&self) -> String;
}

/// Encodes possible measurements for every type of sensor on the vehicle.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
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
#[serde(rename_all = "snake_case")]
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

/// Represents all possible channel types that may be used in a `NodeMapping`.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ChannelType {
	/// A current loop snesor, such as a PT.
	CurrentLoop,

	/// The voltage present on a pin connected to a valve.
	ValveVoltage,

	/// The current flowing through a pin connected to a valve.
	ValveCurrent,

	/// The voltage on the power rail of the board.
	RailVoltage,

	/// The current flowing through the power rail of the board.
	RailCurrent,

	/// The signal carried by a differential pair.
	DifferentialSignal,

	/// The channel of a resistance thermometer, measuring temperature.
	Rtd,

	/// The channel of a thermocouple, measuring temperature.
	Tc,
}

impl ToSql for ChannelType {
	fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
		// listen, I hate this.
		// this works by serializing the ChannelType to JSON and then cutting off
		// the quotation marks at each end. I do not like this, and nor should you.
		// however, this is the final result after a while of debating whether to
		// do match arms for each case, which would be annoying for future capability
		// purposes. so this one won. this should not affect performance, but I hate it
		// nonetheless. FromSql is implemented in a similarly terrible way.
		let mut json = serde_json::to_string(&self)
			.expect("failed to serialize ChannelType into JSON (this should not be possible)");

		json.pop();
		json.remove(0);
		

		Ok(ToSqlOutput::Owned(rusqlite::types::Value::Text(json)))
	}
}

impl FromSql for ChannelType {
	fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
		if let ValueRef::Text(text) = value {
			// see the ToSql comment for details
			let mut json = vec![b'"'];
			json.extend_from_slice(text);
			json.push(b'"');

			let channel_type = serde_json::from_slice(&json)	
				.map_err(|error| FromSqlError::Other(Box::new(error)))?;

			Ok(channel_type)
		} else {
			Err(FromSqlError::InvalidType)
		}
	}
}

/// Used in a `NodeMapping` to determine which computer the action should be send to.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Computer {
	/// The flight computer
	Flight,

	/// The ground computer
	Ground,
}

impl ToSql for Computer {
	fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
		// see the ChannelType ToSql comment for details
		let mut json = serde_json::to_string(&self)
			.expect("failed to serialize ChannelType into JSON (this should not be possible)");

		json.pop();
		json.remove(0);

		Ok(ToSqlOutput::Owned(rusqlite::types::Value::Text(json)))
	}
}

impl FromSql for Computer {
	fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
		if let ValueRef::Text(text) = value {
			// see the ChannelType ToSql comment for details
			let mut json = vec![b'"'];
			json.extend_from_slice(text);
			json.push(b'"');

			let channel_type = serde_json::from_slice(&json)	
				.map_err(|error| FromSqlError::Other(Box::new(error)))?;

			Ok(channel_type)
		} else {
			Err(FromSqlError::InvalidType)
		}
	}
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

/// A sequence written in Python, used by the flight computer to execute arbitrary operator code.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Sequence {
	/// The unique, human-readable name which identifies the sequence.
	///
	/// If the name is "abort" specifically, the sequence should be stored by the recipient and
	/// persisted across a machine power-down instead of run immediately.
	pub name: String,

	/// The script run immediately (except abort) upon being received. 
	pub script: String,
}

/// A trigger with a
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Trigger {
	/// The unique, human-readable name which identifies the trigger.
	pub name: String,

	/// The condition upon which the trigger script is run, written in Python.
	pub condition: String,

	/// The script run when the condition is met, written in Python.
	pub script: String,
}

/// A message sent from the control server to the flight computer.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum ControlMessage {
	/// A set of mappings to be applied immediately.
	Mappings(Vec<NodeMapping>),

	/// A message containing a sequence to be run immediately.
	Sequence(Sequence),

	/// A trigger to be checked by the flight computer.
	Trigger(Trigger),
}
