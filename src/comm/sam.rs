use serde::{Deserialize, Serialize};
use super::ChannelType;

/// A control message send from the flight computer to a SAM board.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum SamControlMessage {
	/// Instructs the board to actuate a valve.
	ActuateValve {
		/// The channel that the valve is connected to.
		channel: u32,

		/// Set to `true` for open and `false` for close.
		open: bool,
	},
	/// Instructs the board to set an LED.
	SetLed {
		/// The channel that the LED is wired to.
		channel: u32,

		/// Set to `true` to turn off and `false` to turn off.
		on: bool,
	}
}

/// A single data point with a timestamp and channel, no units.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct DataPoint {
	/// The raw float value of the measurement, no units.
	pub value: f64,
	
	/// The exact UNIX timestamp of when this single data point was recorded.
	pub timestamp: f64,

	/// The channel that the data point was recorded from.
	pub channel: u32,

	/// The channel 
	pub channel_type: ChannelType,
}

/// A generic data message that can originate from any subsystem.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum DataMessage {
	/// An array of channel data points.
	Sam(Vec<DataPoint>),
	
	/// Data originating from the BMS.
	Bms,
}
