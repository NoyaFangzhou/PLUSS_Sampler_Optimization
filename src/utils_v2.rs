use std::arch::asm;
use std::collections::HashMap;
use std::ops::Deref;
use std::sync::{Arc, Mutex};
use lazy_static::lazy_static;
use statrs::distribution::{Discrete, NegativeBinomial};
use dashmap::DashMap;

type Histogram = HashMap<i64, f64>;
type CHistogram = DashMap<i64, f64>;

const THREAD_NUM: usize = 4;
const POLYBENCH_CACHE_SIZE_KB: usize = 2560;

lazy_static! {
    pub static ref _NoSharePRI: CHistogram = Default::default();
    static ref _RIHist: CHistogram = CHistogram::new();
    static ref _MRC: DashMap<u64, f64> = DashMap::new();
    // static ref _NoSharePRI: Vec<Histogram> = vec![Histogram::new(); THREAD_NUM];
    static ref _SharePRI: CHistogram = Default::default(); //FIXME: i changed the inner hashmap to i64, Histogram instead of i32, Histogram
}

pub(crate) fn pluss_cri_share_histogram_update(tid: usize, share_ratio: i32, reuse: i64, cnt: f64) {
	let in_log_format = false;
    let mut local_reuse = reuse;
    if local_reuse > 0 && in_log_format {
        local_reuse = _polybench_to_highest_power_of_two(local_reuse);
    }
    if _SharePRI.contains_key(&local_reuse) {
		*_SharePRI.get_mut(&local_reuse).unwrap() += cnt;
        // *histogram[tid].get_mut(&reuse).unwrap() += cnt; // this does the same thing as above
    } else {
        _SharePRI.insert(local_reuse, cnt);
    }
    // println!("histogram: {:?}", histogram);
}

pub(crate) fn pluss_cri_noshare_histogram_update(tid: usize, reuse: i64, cnt: f64, in_log_format: Option<bool>) {
    let in_log_format = in_log_format.unwrap_or(false);
    let mut local_reuse = reuse;
    if local_reuse > 0 && in_log_format {
        local_reuse = _polybench_to_highest_power_of_two(local_reuse);
    }
    if _NoSharePRI.contains_key(&local_reuse) {
        *_NoSharePRI.get_mut(&local_reuse).unwrap() += cnt;
        // *histogram[tid].get_mut(&reuse).unwrap() += cnt; // this does the same thing as above
    } else {
        _NoSharePRI.insert(local_reuse, cnt);
    }
    // println!("histogram: {:?}", histogram);
}

pub(crate) fn _polybench_to_highest_power_of_two(mut x: i64) -> i64 {
    // Check for the set bits
    x |= x >> 1;
    x |= x >> 2;
    x |= x >> 4;
    x |= x >> 8;
    x |= x >> 16;
    x |= x >> 32;
    // Then we remove all but the top bit by xor'ing the
    // string of 1's with that string of 1's
    // shifted one to the left, and we end up with
    // just the one top bit followed by 0's
    x ^ (x >> 1)
}

pub(crate) fn _pluss_histogram_print(title: &str, histogram: &CHistogram) {
    println!("{}", title);
    let mut sum: f64 = 0.0;
    // for (k, v) in histogram {
    //     println!("{},{},{}", k, v, v / sum);
    // }
	for it in histogram.iter() {
        let v = it.value();
        sum += v;
    }
	histogram.iter().for_each(|it| println!("{},{},{}", it.key(), it.value(), it.value() / sum))
}

pub(crate) fn pluss_cri_noshare_print_histogram() {
    _pluss_histogram_print("Start to dump noshare private reuse time", &_NoSharePRI);
}

pub(crate) fn pluss_cri_share_print_histogram() {
    _pluss_histogram_print("Start to dump share private reuse time", &_SharePRI);
}

pub(crate) fn pluss_print_histogram() {
    _pluss_histogram_print("Start to dump reuse time", &_RIHist);
}