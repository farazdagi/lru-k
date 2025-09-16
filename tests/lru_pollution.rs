mod common;

use std::num::NonZeroUsize;

use common::*;
use lru::LruCache;
use pretty_assertions::assert_eq;

use crate::common::Runner;

#[test]
fn lru_cache() {
    let mut cache = LruCache::<u64, ()>::new(NonZeroUsize::new(CACHE_CAPACITY).unwrap());

    let mut runner = Runner::new(&mut cache, 42);

    // warm-up cache
    let metrics = runner.access_hot(NUM_HOT_ITEMS, 1_000_000);
    // all hot items are cached, so only first accesses are misses
    assert_eq!(metrics.hit_rate(), 0.99);

    // ensure that all items are cached
    let metrics = runner.access_hot(10_000, 10_000);
    // all hot items are cached
    assert_eq!(metrics.hit_rate(), 1.);

    // run cold scan, on items outside the hot items range
    let metrics = runner.scan_cold(10_000, 10_000);
    // none of the items in cache
    assert_eq!(metrics.hit_rate(), 0.);

    // re-try hot items set, cache should be polluted
    let metrics = runner.access_hot(10_000, 10_000);
    // From 100% hot items being in cache, drop to less than 40% hit rate
    // (effectively meaning that cache gets repopulated with hot items).
    // The LRU-k implementation, will survive such polluting scan without
    // significant amount of unnecessary evictions.
    assert!(metrics.hit_rate() < 0.4);
}
