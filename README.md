# fast-async-mutex
[![](https://github.com/Mnwa/fast-async-mutex/workflows/build/badge.svg?branch=master)](https://github.com/Mnwa/fast-async-mutex/actions?query=workflow%3Abuild)
[![](https://docs.rs/fast-async-mutex/badge.svg)](https://docs.rs/fast-async-mutex/)
[![](https://img.shields.io/crates/v/fast-async-mutex.svg)](https://crates.io/crates/fast-async-mutex)
[![](https://img.shields.io/crates/d/fast-async-mutex.svg)](https://crates.io/crates/fast-async-mutex)

It is a lib which provide asynchronous locking mechanisms, which used spinlock algorithm.
It's maybe very efficient because when mutex tries to acquire data unsuccessfully, these returning control to an async runtime back.
This lib built only on atomics and don't use others std synchronous data structures, which make this lib so fast.

## Examples

```rust
use fast_async_mutex::mutex::Mutex;

#[tokio::main]
async fn main() {
    let mutex = Mutex::new(10);
    let guard = mutex.lock().await;
    assert_eq!(*guard, 10);
}
```

## Benchmarks

There is result of benchmarks which runs on `MacBook Pro (16-inch, 2019) 2,3 GHz 8-Core Intel Core i9 16GB RAM`
Tests you can find in the [benchmarks dir](benchmarks).
```
running 35 tests
test mutex::fast_async_mutex::tests::concurrency_without_waiting          ... bench:   1,527,148 ns/iter (+/- 146,486)
test mutex::fast_async_mutex::tests::create                               ... bench:           0 ns/iter (+/- 0)
test mutex::fast_async_mutex::tests::step_by_step_without_waiting         ... bench:       2,529 ns/iter (+/- 157)
test mutex::fast_async_mutex_ordered::tests::concurrency_without_waiting  ... bench:   1,531,361 ns/iter (+/- 150,229)
test mutex::fast_async_mutex_ordered::tests::create                       ... bench:           0 ns/iter (+/- 0)
test mutex::fast_async_mutex_ordered::tests::step_by_step_without_waiting ... bench:       3,061 ns/iter (+/- 434)
test mutex::futures::tests::concurrency_without_waiting                   ... bench:   1,541,129 ns/iter (+/- 141,631)
test mutex::futures::tests::create                                        ... bench:          91 ns/iter (+/- 12)
test mutex::futures::tests::step_by_step_without_waiting                  ... bench:       2,969 ns/iter (+/- 353)
test mutex::smol::tests::concurrency_without_waiting                      ... bench:   1,548,987 ns/iter (+/- 170,713)
test mutex::smol::tests::create                                           ... bench:           0 ns/iter (+/- 0)
test mutex::smol::tests::step_by_step_without_waiting                     ... bench:       3,788 ns/iter (+/- 416)
test mutex::tokio::tests::concurrency_without_waiting                     ... bench:   1,545,367 ns/iter (+/- 125,671)
test mutex::tokio::tests::create                                          ... bench:           5 ns/iter (+/- 0)
test mutex::tokio::tests::step_by_step_without_waiting                    ... bench:       6,818 ns/iter (+/- 157)
test rwlock::fast_async_mutex::tests::concurrency_read                    ... bench:   1,544,827 ns/iter (+/- 127,652)
test rwlock::fast_async_mutex::tests::concurrency_write                   ... bench:   1,544,193 ns/iter (+/- 301,309)
test rwlock::fast_async_mutex::tests::create                              ... bench:           0 ns/iter (+/- 0)
test rwlock::fast_async_mutex::tests::step_by_step_read                   ... bench:       3,707 ns/iter (+/- 779)
test rwlock::fast_async_mutex::tests::step_by_step_writing                ... bench:       2,597 ns/iter (+/- 183)
test rwlock::fast_async_mutex_ordered::tests::concurrency_read            ... bench:   1,571,000 ns/iter (+/- 114,008)
test rwlock::fast_async_mutex_ordered::tests::concurrency_write           ... bench:   1,560,389 ns/iter (+/- 115,898)
test rwlock::fast_async_mutex_ordered::tests::create                      ... bench:           0 ns/iter (+/- 0)
test rwlock::fast_async_mutex_ordered::tests::step_by_step_read           ... bench:       3,639 ns/iter (+/- 158)
test rwlock::fast_async_mutex_ordered::tests::step_by_step_writing        ... bench:       2,888 ns/iter (+/- 164)
test rwlock::smol::tests::concurrency_read                                ... bench:   1,542,633 ns/iter (+/- 171,292)
test rwlock::smol::tests::concurrency_write                               ... bench:   1,553,628 ns/iter (+/- 180,358)
test rwlock::smol::tests::create                                          ... bench:           1 ns/iter (+/- 0)
test rwlock::smol::tests::step_by_step_read                               ... bench:       3,184 ns/iter (+/- 111)
test rwlock::smol::tests::step_by_step_writing                            ... bench:       5,880 ns/iter (+/- 911)
test rwlock::tokio::tests::concurrency_read                               ... bench:   1,544,295 ns/iter (+/- 112,044)
test rwlock::tokio::tests::concurrency_write                              ... bench:   1,543,137 ns/iter (+/- 146,681)
test rwlock::tokio::tests::create                                         ... bench:           5 ns/iter (+/- 0)
test rwlock::tokio::tests::step_by_step_read                              ... bench:       6,530 ns/iter (+/- 382)
test rwlock::tokio::tests::step_by_step_writing                           ... bench:       6,576 ns/iter (+/- 283)

test result: ok. 0 passed; 0 failed; 0 ignored; 35 measured; 0 filtered out
```

## License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

#### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
