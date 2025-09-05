use {
    lru::LruCache,
    std::{num::NonZeroUsize, time::Instant},
};

#[derive(Clone, Copy)]
struct Metrics {
    hits: u64,
    misses: u64,
}

impl Metrics {
    fn new() -> Self {
        Self { hits: 0, misses: 0 }
    }

    fn record_hit(&mut self) {
        self.hits += 1;
    }

    fn record_miss(&mut self) {
        self.misses += 1;
    }

    fn total(&self) -> u64 {
        self.hits + self.misses
    }

    fn hit_rate(&self) -> f64 {
        let t = self.total();
        if t == 0 {
            0.0
        } else {
            self.hits as f64 / t as f64
        }
    }
}

// Tiny LCG so we avoid extra deps.
struct Lcg(u64);
impl Lcg {
    fn new(seed: u64) -> Self {
        Self(seed)
    }

    fn next_u64(&mut self) -> u64 {
        // 64-bit LCG constants
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1);
        self.0
    }

    fn gen_range(&mut self, n: u64) -> u64 {
        if n == 0 {
            0
        } else {
            self.next_u64() % n
        }
    }
}

fn access_hot(
    cache: &mut LruCache<u64, ()>,
    hot_size: usize,
    ops: usize,
    rng: &mut Lcg,
) -> Metrics {
    let mut m = Metrics::new();
    for _ in 0..ops {
        let k = rng.gen_range(hot_size as u64) as u64;
        if cache.get(&k).is_some() {
            m.record_hit();
        } else {
            m.record_miss();
            cache.put(k, ());
        }
    }
    m
}

fn scan_cold(cache: &mut LruCache<u64, ()>, hot_size: usize, scan_size: usize) -> Metrics {
    let mut m = Metrics::new();
    for i in 0..scan_size {
        let k = (hot_size + i) as u64;
        if cache.get(&k).is_some() {
            m.record_hit();
        } else {
            m.record_miss();
            cache.put(k, ());
        }
    }
    m
}

fn parse_args() -> (usize, usize, usize, usize, u64) {
    // args: capacity hot_size scan_size hot_ops seed
    // Defaults chosen to clearly show LRU pollution.
    let mut args = std::env::args().skip(1);
    let cap = args.next().and_then(|s| s.parse().ok()).unwrap_or(1024);
    let hot = args.next().and_then(|s| s.parse().ok()).unwrap_or(1024);
    let scan = args.next().and_then(|s| s.parse().ok()).unwrap_or(50_000);
    let hot_ops = args.next().and_then(|s| s.parse().ok()).unwrap_or(20_000);
    let seed = args
        .next()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0xC0FFEEu64);
    (cap, hot, scan, hot_ops, seed)
}

fn main() {
    let (capacity, hot_size, scan_size, hot_ops, seed) = parse_args();

    if capacity == 0 || hot_size == 0 {
        eprintln!("capacity and hot_size must be > 0");
        std::process::exit(2);
    }
    if scan_size == 0 {
        eprintln!("scan_size must be > 0");
        std::process::exit(2);
    }

    println!("LRU scan-resistance demo");
    println!("capacity      = {}", capacity);
    println!("hot_size      = {}", hot_size);
    println!("scan_size     = {}", scan_size);
    println!("hot_ops       = {}", hot_ops);
    println!("seed          = {}", seed);
    if capacity != hot_size {
        println!("Note: setting capacity == hot_size makes the effect starkest.");
    }

    let cap_nz = NonZeroUsize::new(capacity).unwrap();
    let mut cache = LruCache::<u64, ()>::new(cap_nz);
    let mut rng = Lcg::new(seed);

    // Warm-up: teach the cache the hot set well.
    let warm_ops = hot_ops.max(hot_size * 2);
    let t0 = Instant::now();
    let warm = access_hot(&mut cache, hot_size, warm_ops, &mut rng);
    let d0 = t0.elapsed();

    println!(
        "Warm-up: hot accesses = {}, hit_rate = {:.2}%, time = {:?}",
        warm.total(),
        warm.hit_rate() * 100.0,
        d0
    );

    // Snapshot pre-scan hot performance over a bounded window.
    let pre = access_hot(&mut cache, hot_size, hot_ops, &mut rng);
    println!(
        "Pre-scan hot window: hit_rate = {:.2}%",
        pre.hit_rate() * 100.0
    );

    // One-pass cold scan that exceeds capacity.
    let t1 = Instant::now();
    let cold = scan_cold(&mut cache, hot_size, scan_size);
    let d1 = t1.elapsed();
    println!(
        "Cold scan: items = {}, hits = {}, misses = {}, time = {:?}",
        scan_size, cold.hits, cold.misses, d1
    );

    // Immediately after the scan, measure hot again.
    let post = access_hot(&mut cache, hot_size, hot_ops, &mut rng);
    println!(
        "Post-scan hot window: hit_rate = {:.2}%",
        post.hit_rate() * 100.0
    );

    // Simple recovery estimate: how many ops until we exceed 90% hit rate again?
    let target_rate = 0.90;
    let mut rec_cache = cache; // continue from current state
    let mut rec_rng = rng;
    let mut hits = 0u64;
    let mut misses = 0u64;
    let mut steps_to_recover: Option<u64> = None;

    for i in 0..(hot_size * 10) {
        let k = rec_rng.gen_range(hot_size as u64) as u64;
        if rec_cache.get(&k).is_some() {
            hits += 1;
        } else {
            misses += 1;
            rec_cache.put(k, ());
        }

        let total = hits + misses;
        if total >= 100 {
            // need a bit of window to stabilize
            let hr = hits as f64 / total as f64;
            if hr >= target_rate {
                steps_to_recover = Some(total);
                break;
            }
        }

        // stop if we ran enough without recovery
        if i + 1 == hot_size * 10 {
            steps_to_recover = None;
        }
    }

    match steps_to_recover {
        Some(n) => println!(
            "Recovery: ~{n} hot ops needed to exceed {:.0}% hit rate again.",
            target_rate * 100.0
        ),
        None => println!(
            "Recovery: did not exceed {:.0}% hit rate within {} ops.",
            target_rate * 100.0,
            hot_size * 10
        ),
    }

    println!("\nInterpretation:");
    println!("- Pre-scan: hot hit-rate should be high (near 100% if capacity >= hot_size).");
    println!("- After one cold scan: hot hit-rate collapses (LRU polluted by single-touch keys).");
    println!("- Recovery shows extra work needed just to get back to a healthy hit-rate.");
}
