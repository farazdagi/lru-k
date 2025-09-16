use std::fmt;

use lru::LruCache;
use rand::{
    Rng,
    SeedableRng,
    rngs::SmallRng,
};

pub const NUM_HOT_ITEMS: usize = 10_000;
pub const CACHE_CAPACITY: usize = NUM_HOT_ITEMS;

pub struct Metrics {
    hits: u64,
    misses: u64,
}

impl fmt::Debug for Metrics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Metrics")
            .field("hits", &self.hits)
            .field("misses", &self.misses)
            .field("hit_rate", &self.hit_rate())
            .finish()
    }
}

impl Metrics {
    pub fn new() -> Self {
        Self { hits: 0, misses: 0 }
    }

    pub fn record_hit(&mut self) {
        self.hits += 1;
    }

    pub fn record_miss(&mut self) {
        self.misses += 1;
    }

    pub fn total(&self) -> u64 {
        self.hits + self.misses
    }

    pub fn hit_rate(&self) -> f64 {
        let t = self.total();
        if t == 0 {
            0.0
        } else {
            self.hits as f64 / t as f64
        }
    }
}

pub struct Runner<'a> {
    cache: &'a mut LruCache<u64, ()>,
    rng: SmallRng,
}

impl<'a> Runner<'a> {
    pub fn new(cache: &'a mut LruCache<u64, ()>, seed: u64) -> Self {
        Self {
            cache,
            rng: SmallRng::seed_from_u64(seed),
        }
    }

    pub fn access_hot(&mut self, hot_size: usize, ops: usize) -> Metrics {
        let mut m = Metrics::new();
        for _ in 0..ops {
            let k = self.rng.random_range(0..hot_size) as u64;
            if self.cache.get(&k).is_some() {
                m.record_hit();
            } else {
                m.record_miss();
                self.cache.put(k, ());
            }
        }
        m
    }

    pub fn scan_cold(&mut self, hot_size: usize, scan_size: usize) -> Metrics {
        let mut m = Metrics::new();
        for i in 0..scan_size {
            // All scanned items are outside of hot range.
            let k = (hot_size + i) as u64;
            if self.cache.get(&k).is_some() {
                m.record_hit();
            } else {
                m.record_miss();
                self.cache.put(k, ());
            }
        }
        m
    }
}
