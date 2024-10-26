#[derive(Debug, Clone)]
pub struct IdQueue {
	queue: Vec<u32>
}

impl IdQueue {
	pub fn new() -> Self {
		Self { queue: Vec::new() }
	}

	pub fn from_vec(queue: Vec<u32>) -> Self {
		Self { queue }
	}

	pub fn front(&self) -> Option<u32> {
		Some(self.queue[0])
	}

	pub fn enqueue(&mut self, id: u32) {
		self.queue.push(id)
	}

	pub fn sorted_enqueue(&mut self, id: u32) {
		self.enqueue(id);
		self.queue.sort();
	}

	pub fn dequeue(&mut self) -> Option<u32> {
		Some(self.queue.remove(0))

	}

	pub fn is_empty(&self) -> bool {
		self.queue.len() == 0
	}

	pub fn size(&self) -> u32 {
		self.queue.len() as u32
	}
}
