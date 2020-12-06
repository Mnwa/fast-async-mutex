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
test mutex::fast_async_mutex::tests::concurrency_without_waiting          ... bench:      47,696 ns/iter (+/- 1,623)
test mutex::fast_async_mutex::tests::create                               ... bench:           0 ns/iter (+/- 0)
test mutex::fast_async_mutex::tests::step_by_step_without_waiting         ... bench:      22,306 ns/iter (+/- 10,980)
test mutex::fast_async_mutex_ordered::tests::concurrency_without_waiting  ... bench:      51,407 ns/iter (+/- 5,661)
test mutex::fast_async_mutex_ordered::tests::create                       ... bench:           0 ns/iter (+/- 0)
test mutex::fast_async_mutex_ordered::tests::step_by_step_without_waiting ... bench:      24,749 ns/iter (+/- 5,132)
test mutex::futures::tests::concurrency_without_waiting                   ... bench:      73,485 ns/iter (+/- 7,469)
test mutex::futures::tests::create                                        ... bench:          94 ns/iter (+/- 31)
test mutex::futures::tests::step_by_step_without_waiting                  ... bench:      23,365 ns/iter (+/- 6,377)
test mutex::smol::tests::concurrency_without_waiting                      ... bench:      65,972 ns/iter (+/- 4,398)
test mutex::smol::tests::create                                           ... bench:           0 ns/iter (+/- 0)
test mutex::smol::tests::step_by_step_without_waiting                     ... bench:      26,291 ns/iter (+/- 10,595)
test mutex::tokio::tests::concurrency_without_waiting                     ... bench:      59,337 ns/iter (+/- 6,142)
test mutex::tokio::tests::create                                          ... bench:           5 ns/iter (+/- 0)
test mutex::tokio::tests::step_by_step_without_waiting                    ... bench:      27,369 ns/iter (+/- 6,044)
test rwlock::fast_async_mutex::tests::concurrency_read                    ... bench:      88,433 ns/iter (+/- 245,452)
test rwlock::fast_async_mutex::tests::concurrency_write                   ... bench:      55,364 ns/iter (+/- 136,763)
test rwlock::fast_async_mutex::tests::create                              ... bench:           0 ns/iter (+/- 0)
test rwlock::fast_async_mutex::tests::step_by_step_read                   ... bench:      22,084 ns/iter (+/- 1,721)
test rwlock::fast_async_mutex::tests::step_by_step_writing                ... bench:      20,935 ns/iter (+/- 3,714)
test rwlock::fast_async_mutex_ordered::tests::concurrency_read            ... bench:      49,386 ns/iter (+/- 1,318)
test rwlock::fast_async_mutex_ordered::tests::concurrency_write           ... bench:      51,376 ns/iter (+/- 1,166)
test rwlock::fast_async_mutex_ordered::tests::create                      ... bench:           0 ns/iter (+/- 0)
test rwlock::fast_async_mutex_ordered::tests::step_by_step_read           ... bench:      22,400 ns/iter (+/- 1,909)
test rwlock::fast_async_mutex_ordered::tests::step_by_step_writing        ... bench:      22,207 ns/iter (+/- 4,611)
test rwlock::smol::tests::concurrency_read                                ... bench:      52,462 ns/iter (+/- 2,141)
test rwlock::smol::tests::concurrency_write                               ... bench:      82,777 ns/iter (+/- 1,934)
test rwlock::smol::tests::create                                          ... bench:           2 ns/iter (+/- 0)
test rwlock::smol::tests::step_by_step_read                               ... bench:      22,378 ns/iter (+/- 1,945)
test rwlock::smol::tests::step_by_step_writing                            ... bench:      25,467 ns/iter (+/- 2,545)
test rwlock::tokio::tests::concurrency_read                               ... bench:      58,172 ns/iter (+/- 905)
test rwlock::tokio::tests::concurrency_write                              ... bench:      57,282 ns/iter (+/- 2,358)
test rwlock::tokio::tests::create                                         ... bench:           5 ns/iter (+/- 0)
test rwlock::tokio::tests::step_by_step_read                              ... bench:      26,509 ns/iter (+/- 7,007)
test rwlock::tokio::tests::step_by_step_writing                           ... bench:      26,021 ns/iter (+/- 6,407)

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
