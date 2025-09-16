//! A fixed-size thread safe LRU-k cache.
//!
//! The design is based on the [The LRU-K page replacement algorithm for database disk buffering](https://dl.acm.org/doi/pdf/10.1145/170036.170081) paper.

#![no_std]

#[cfg(test)]
#[macro_use]
extern crate std;

mod list;

type Id = u32;
