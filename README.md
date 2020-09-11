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
test mutex::fast_async_mutex::tests::concurrency_without_waiting          ... bench:   1,385,589 ns/iter (+/- 276,322)
test mutex::fast_async_mutex::tests::create                               ... bench:           0 ns/iter (+/- 0)
test mutex::fast_async_mutex::tests::step_by_step_without_waiting         ... bench:       3,044 ns/iter (+/- 394)
test mutex::fast_async_mutex_ordered::tests::concurrency_without_waiting  ... bench:   1,515,206 ns/iter (+/- 508,395)
test mutex::fast_async_mutex_ordered::tests::create                       ... bench:           0 ns/iter (+/- 0)
test mutex::fast_async_mutex_ordered::tests::step_by_step_without_waiting ... bench:       3,581 ns/iter (+/- 664)
test mutex::futures::tests::concurrency_without_waiting                   ... bench:   1,522,057 ns/iter (+/- 242,784)
test mutex::futures::tests::create                                        ... bench:          90 ns/iter (+/- 11)
test mutex::futures::tests::step_by_step_without_waiting                  ... bench:       3,392 ns/iter (+/- 684)
test mutex::smol::tests::concurrency_without_waiting                      ... bench:   1,484,722 ns/iter (+/- 192,105)
test mutex::smol::tests::create                                           ... bench:           0 ns/iter (+/- 0)
test mutex::smol::tests::step_by_step_without_waiting                     ... bench:       3,548 ns/iter (+/- 581)
test mutex::tokio::tests::concurrency_without_waiting                     ... bench:   1,475,216 ns/iter (+/- 377,972)
test mutex::tokio::tests::create                                          ... bench:          89 ns/iter (+/- 17)
test mutex::tokio::tests::step_by_step_without_waiting                    ... bench:       7,702 ns/iter (+/- 784)
test rwlock::fast_async_mutex::tests::concurrency_read                    ... bench:   1,535,692 ns/iter (+/- 340,077)
test rwlock::fast_async_mutex::tests::concurrency_write                   ... bench:   1,535,635 ns/iter (+/- 382,335)
test rwlock::fast_async_mutex::tests::create                              ... bench:           0 ns/iter (+/- 0)
test rwlock::fast_async_mutex::tests::step_by_step_read                   ... bench:       3,771 ns/iter (+/- 511)
test rwlock::fast_async_mutex::tests::step_by_step_writing                ... bench:       3,135 ns/iter (+/- 674)
test rwlock::fast_async_mutex_ordered::tests::concurrency_read            ... bench:   1,390,001 ns/iter (+/- 237,342)
test rwlock::fast_async_mutex_ordered::tests::concurrency_write           ... bench:   1,433,193 ns/iter (+/- 862,141)
test rwlock::fast_async_mutex_ordered::tests::create                      ... bench:           0 ns/iter (+/- 0)
test rwlock::fast_async_mutex_ordered::tests::step_by_step_read           ... bench:       4,296 ns/iter (+/- 771)
test rwlock::fast_async_mutex_ordered::tests::step_by_step_writing        ... bench:       3,149 ns/iter (+/- 284)
test rwlock::smol::tests::concurrency_read                                ... bench:   1,482,705 ns/iter (+/- 1,124,641)
test rwlock::smol::tests::concurrency_write                               ... bench:   1,471,196 ns/iter (+/- 163,698)
test rwlock::smol::tests::create                                          ... bench:           1 ns/iter (+/- 0)
test rwlock::smol::tests::step_by_step_read                               ... bench:       4,131 ns/iter (+/- 1,250)
test rwlock::smol::tests::step_by_step_writing                            ... bench:       5,681 ns/iter (+/- 1,489)
test rwlock::tokio::tests::concurrency_read                               ... bench:   1,413,247 ns/iter (+/- 191,029)
test rwlock::tokio::tests::concurrency_write                              ... bench:   1,409,945 ns/iter (+/- 110,948)
test rwlock::tokio::tests::create                                         ... bench:          84 ns/iter (+/- 13)
test rwlock::tokio::tests::step_by_step_read                              ... bench:       8,429 ns/iter (+/- 1,061)
test rwlock::tokio::tests::step_by_step_writing                           ... bench:       8,420 ns/iter (+/- 967)

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
