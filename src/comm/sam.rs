use serde::{Deserialize, Serialize};
use super::ChannelType;

/// A control message send from the flight computer to a SAM board.
#[derive(Clone, Debug, Deserialize, Serialize)]
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

/// A single raw data point without a unit but with a timestamp.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RawDataPoint {
	/// The raw float value of the measurement, no units.
	pub value: f64,
	
	/// The exact UNIX timestamp of when this single data point was recorded.
	pub timestamp: f64,
}

/// A burst of channel data, typically sent from a SAM board.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ChannelDataBurst {
	/// The channel that the data was recorded from.
	pub channel: u32,

	/// The channel type (implying unit) that the data is from.
	pub channel_type: ChannelType,
	
	/// The array of raw data points recorded from the channel.
	pub data_points: Vec<RawDataPoint>,
}

/// A generic data message that can originate from any subsystem.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum DataMessage {
	/// An array of channel data bursts containing arrays of raw data points.
	Sam(Vec<ChannelDataBurst>),
	
	/// Data originating from the BMS.
	Bms,
}
