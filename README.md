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
test mutex::fast_async_mutex::tests::concurrency_without_waiting          ... bench:   1,491,590 ns/iter (+/- 245,669)
test mutex::fast_async_mutex::tests::create                               ... bench:           0 ns/iter (+/- 0)
test mutex::fast_async_mutex::tests::step_by_step_without_waiting         ... bench:       3,063 ns/iter (+/- 412)
test mutex::fast_async_mutex_ordered::tests::concurrency_without_waiting  ... bench:   1,477,564 ns/iter (+/- 247,044)
test mutex::fast_async_mutex_ordered::tests::create                       ... bench:           0 ns/iter (+/- 0)
test mutex::fast_async_mutex_ordered::tests::step_by_step_without_waiting ... bench:       3,215 ns/iter (+/- 477)
test mutex::futures::tests::concurrency_without_waiting                   ... bench:   1,493,409 ns/iter (+/- 273,137)
test mutex::futures::tests::create                                        ... bench:          92 ns/iter (+/- 22)
test mutex::futures::tests::step_by_step_without_waiting                  ... bench:       3,419 ns/iter (+/- 589)
test mutex::smol::tests::concurrency_without_waiting                      ... bench:   1,498,358 ns/iter (+/- 249,535)
test mutex::smol::tests::create                                           ... bench:           0 ns/iter (+/- 0)
test mutex::smol::tests::step_by_step_without_waiting                     ... bench:       3,594 ns/iter (+/- 509)
test mutex::tokio::tests::concurrency_without_waiting                     ... bench:   1,510,567 ns/iter (+/- 256,648)
test mutex::tokio::tests::create                                          ... bench:          88 ns/iter (+/- 12)
test mutex::tokio::tests::step_by_step_without_waiting                    ... bench:       7,461 ns/iter (+/- 989)
test rwlock::fast_async_mutex::tests::concurrency_read                    ... bench:   1,531,031 ns/iter (+/- 291,727)
test rwlock::fast_async_mutex::tests::concurrency_write                   ... bench:   1,488,366 ns/iter (+/- 229,080)
test rwlock::fast_async_mutex::tests::create                              ... bench:           0 ns/iter (+/- 0)
test rwlock::fast_async_mutex::tests::step_by_step_read                   ... bench:       3,553 ns/iter (+/- 591)
test rwlock::fast_async_mutex::tests::step_by_step_writing                ... bench:       2,899 ns/iter (+/- 174)
test rwlock::fast_async_mutex_ordered::tests::concurrency_read            ... bench:   1,546,864 ns/iter (+/- 245,088)
test rwlock::fast_async_mutex_ordered::tests::concurrency_write           ... bench:   1,477,071 ns/iter (+/- 242,150)
test rwlock::fast_async_mutex_ordered::tests::create                      ... bench:           0 ns/iter (+/- 0)
test rwlock::fast_async_mutex_ordered::tests::step_by_step_read           ... bench:       4,333 ns/iter (+/- 312)
test rwlock::fast_async_mutex_ordered::tests::step_by_step_writing        ... bench:       3,425 ns/iter (+/- 1,652)
test rwlock::smol::tests::concurrency_read                                ... bench:   1,466,957 ns/iter (+/- 240,481)
test rwlock::smol::tests::concurrency_write                               ... bench:   1,482,657 ns/iter (+/- 242,227)
test rwlock::smol::tests::create                                          ... bench:           1 ns/iter (+/- 0)
test rwlock::smol::tests::step_by_step_read                               ... bench:       3,561 ns/iter (+/- 624)
test rwlock::smol::tests::step_by_step_writing                            ... bench:       5,797 ns/iter (+/- 1,426)
test rwlock::tokio::tests::concurrency_read                               ... bench:   1,490,786 ns/iter (+/- 272,406)
test rwlock::tokio::tests::concurrency_write                              ... bench:   1,533,387 ns/iter (+/- 378,954)
test rwlock::tokio::tests::create                                         ... bench:          86 ns/iter (+/- 13)
test rwlock::tokio::tests::step_by_step_read                              ... bench:       8,227 ns/iter (+/- 992)
test rwlock::tokio::tests::step_by_step_writing                           ... bench:       8,645 ns/iter (+/- 2,318)

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
