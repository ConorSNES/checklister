use std::hash::{Hash, Hasher};

use criterion::{criterion_group, criterion_main, Criterion};
use checklister::app::{model::{_make_sample_set}, xorhasher::{XorHasher}};

pub fn criterion_benchmark(c: &mut Criterion) {
	c.bench_function(
		"xorhasher_test",
		|b| b.iter(|| {
			let m = _make_sample_set();
			let mut h = XorHasher::default();
			m.hash(&mut h);
			let _ = h.finish();
		})
	);
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);