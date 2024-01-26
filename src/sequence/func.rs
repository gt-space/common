use std::{thread, time::Instant};

use pyo3::{pyfunction, PyAny, PyResult};

use crate::sequence::unit::Duration;

/// A Python-exposed function which waits the thread for the given duration.
#[pyfunction]
pub fn wait_for(duration: &Duration) {
	// TODO: considering using a different way to sleep, possibly sleeping only the GIL?
	thread::sleep(duration.into());
}

/// A Python-exposed function which waits until a condition function is true, given an optional timeout and interval between checking.
#[pyfunction]
pub fn wait_until(condition: &PyAny, timeout: Option<&Duration>, poll_interval: Option<&Duration>) -> PyResult<()> {
	let timeout = timeout.map_or(std::time::Duration::MAX, Into::into);
	let interval = poll_interval.map_or(std::time::Duration::from_millis(10), Into::into);

	let end_time = Instant::now() + timeout;

	while !condition.call0()?.is_true()? && Instant::now() < end_time {
		thread::sleep(interval);
	}

	Ok(())
}
