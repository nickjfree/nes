use std::time::{Duration, Instant};


// 559 us per cycle
const CPU_CYCLE: u128 = 559;


pub struct Clock {
	prev_time: Instant,
	delta: Duration,
}


impl Clock {

	pub fn new() -> Self {
		Self{
			prev_time: Instant::now(),
			delta: Duration::default(),
		}
	}

	// get cycles to emulate
	pub fn get_cycles_past(&mut self) -> u128 {
		let duration = self.prev_time.elapsed();
		self.prev_time = Instant::now();
		self.delta += duration;
		if self.delta.as_nanos() > CPU_CYCLE {
			let cycles = self.delta.as_nanos() / CPU_CYCLE;
			self.delta = Duration::from_nanos((self.delta.as_nanos() % CPU_CYCLE) as u64);
			cycles
		} else {
			0
		}
	}
}