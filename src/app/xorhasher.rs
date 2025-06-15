use std::hash::Hasher;

#[derive(Default, Debug)]
pub struct XorHasher {
	working : u8
}

impl Hasher for XorHasher {
	fn write(&mut self, bytes: &[u8]) {
		for subval in bytes {
			self.working ^= subval;
		}
	}

	fn finish(&self) -> u64 {
		self.working as u64
	}
}