use std::sync::{Arc, Mutex};

use common::sequence;
use pyo3::Python;



#[test]
fn test_interval() {
	let mappings = Arc::new(Mutex::new(Vec::new()));
	sequence::initialize(mappings);

	let script = 
}