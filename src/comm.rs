use crate::ToPrettyString;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt};

#[cfg(feature = "rusqlite")]
use rusqlite::{ToSql, types::{ToSqlOutput, ValueRef, FromSql, FromSqlResult, FromSqlError}};

mod sam;
pub use sam::*;

/// Every unit needed to be passed around in communications, mainly for sensor readings.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Unit {
	/// Current, in amperes.
	Amps,

	/// Pressure, in pounds per square inch.
	Psi,
	
	/// Temperature, in Kelvin.
	Kelvin,

	/// Force, in pounds.
	Pounds,

	/// Electric potential, in volts.
	Volts,
}

impl fmt::Display for Unit {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}", match self {
			Self::Amps => "A",
			Self::Psi => "psi",
			Self::Kelvin => "K",
			Self::Pounds => "lbf",
			Self::Volts => "V",
		})
	}
}

/// Encodes possible measurements for every type of sensor on the vehicle.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub struct Measurement {
	/// The raw value of the measurement, independent of the unit.
	pub value: f64,

	/// The unit of the measurement, independent of the value.
	pub unit: Unit,
}

impl fmt::Display for Measurement {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{} {}", self.value, self.unit)
	}
}

impl ToPrettyString for Measurement {
	fn to_pretty_string(&self) -> String {
		format!("\x1b[1m{:.2} {}\x1b[0m", self.value, self.unit)
	}
}

/// Encodes every possible valve state.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ValveState {
	/// Valve disconnected.
	Disconnected,

	/// Acknowledged open.
	Open,

	/// Acknowledged closed.
	Closed,

	/// Commanded open, but currently closed.
	CommandedOpen,

	/// Commanded closed, but currently open.
	CommandedClosed,
}

impl fmt::Display for ValveState {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}", match self {
			Self::Disconnected => "disconnected",
			Self::Open => "open",
			Self::Closed => "closed",
			Self::CommandedOpen => "commanded open",
			Self::CommandedClosed => "commanded closed",
		})
	}
}

impl ToPrettyString for ValveState {
	/// Converts the valve state into a colored string ready to be displayed on the interface.
	fn to_pretty_string(&self) -> String {
		match self {
			Self::Disconnected => "\x1b[31mdisconnected\x1b[0m",
			Self::Open => "\x1b[32mopen\x1b[0m",
			Self::Closed => "\x1b[32mclosed\x1b[0m",
			Self::CommandedOpen => "\x1b[33mclosed\x1b[0m",
			Self::CommandedClosed => "\x1b[33mopen\x1b[0m",
		}.to_owned()
	}
}

/// Holds the state of the vehicle using `HashMap`s which convert a node's name to its state.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct VehicleState {
	/// Holds the current states of all valves on the vehicle.
	pub valve_states: HashMap<String, ValveState>,

	/// Holds the latest readings of all sensors on the vehicle.
	pub sensor_readings: HashMap<String, Measurement>,

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
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ChannelType {
	/// A pressure transducer, formerly known as CurrentLoop, which measures the pressure of a fluid.
	CurrentLoop,

	/// The voltage present on a pin connected to a valve.
	ValveVoltage,

	/// The current flowing through a pin connected to a valve.
	ValveCurrent,

	/// The voltage on the power rail of the board.
	RailVoltage,

	/// The current flowing through the power rail of the board.
	RailCurrent,

	/// The signal from a load cell, carried by a differential pair.
	DifferentialSignal,

	/// The channel of a resistance thermometer, measuring temperature.
	Rtd,

	/// The channel of a thermocouple, measuring temperature.
	Tc,
}

impl ChannelType {
	/// Gets the associated unit for the given channel type.
	pub fn unit(&self) -> Unit {
		match self {
			Self::CurrentLoop => Unit::Psi,
			Self::ValveVoltage => Unit::Volts,
			Self::ValveCurrent => Unit::Amps,
			Self::RailVoltage => Unit::Volts,
			Self::RailCurrent => Unit::Amps,
			Self::DifferentialSignal => Unit::Pounds,
			Self::Rtd => Unit::Kelvin,
			Self::Tc => Unit::Kelvin,
		}
	}
}

#[cfg(feature = "rusqlite")]
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

#[cfg(feature = "rusqlite")]
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
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Computer {
	/// The flight computer
	Flight,

	/// The ground computer
	Ground,
}

#[cfg(feature = "rusqlite")]
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

#[cfg(feature = "rusqlite")]
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
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct NodeMapping {
	/// The text identifier, or name, of the node.
	pub text_id: String,
	
	/// A string identifying an individual board, corresponding to the hostname sans ".local".
	pub board_id: String,

	/// The channel type of the node, such as "valve".
	pub channel_type: ChannelType,

	/// A number identifying which channel on the SAM board controls the node.
	pub channel: u32,

	/// Which computer controls the SAM board, "flight" or "ground".
	pub computer: Computer,

	// the optional parameters below are only needed for sensors with certain channel types
	// if you're wondering why these are not kept with the ChannelType variants, that is
	// because those variants are passed back from the SAM boards with data measurements.
	// the SAM boards have no access to these factors and even if they did, it would make
	// more sense for them to just convert the measurements directly.
	//
	// tl;dr this is correct and reasonable.

	/// The maximum value reading of the sensor.
	/// This is only used for sensors with channel type CurrentLoop or DifferentialSignal.
	pub max: Option<f64>,

	/// The minimum value reading of the sensor.
	/// This is only used for sensors with channel type CurrentLoop or DifferentialSignal.
	pub min: Option<f64>,

	/// The calibrated offset of the sensor.
	/// This is only used for sensors with channel type PT.
	#[serde(default)]
	pub calibrated_offset: f64,

	/// The threshold, in Amps, at which the valve is considered connected.
	pub connected_threshold: Option<f64>,

	/// The threshold, in Amps, at which the valve is considered powered.
	pub powered_threshold: Option<f64>,

	/// Indicator of whether the valve is normally open or normally closed.
	pub normally_closed: Option<bool>,
}

/// A sequence written in Python, used by the flight computer to execute arbitrary operator code.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
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
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Trigger {
	/// The unique, human-readable name which identifies the trigger.
	pub name: String,

	/// The condition upon which the trigger script is run, written in Python.
	pub condition: String,

	/// The script run when the condition is met, written in Python.
	pub script: String,
}

/// A message sent from the control server to the flight computer.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum FlightControlMessage {
	/// A set of mappings to be applied immediately.
	Mappings(Vec<NodeMapping>),

	/// A message containing a sequence to be run immediately.
	Sequence(Sequence),

	/// A trigger to be checked by the flight computer.
	Trigger(Trigger),
}
