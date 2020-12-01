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
test mutex::fast_async_mutex::tests::concurrency_without_waiting          ... bench:      48,472 ns/iter (+/- 1,635)
test mutex::fast_async_mutex::tests::create                               ... bench:           0 ns/iter (+/- 0)
test mutex::fast_async_mutex::tests::step_by_step_without_waiting         ... bench:      22,710 ns/iter (+/- 2,502)
test mutex::fast_async_mutex_ordered::tests::concurrency_without_waiting  ... bench:      62,054 ns/iter (+/- 2,821)
test mutex::fast_async_mutex_ordered::tests::create                       ... bench:           0 ns/iter (+/- 0)
test mutex::fast_async_mutex_ordered::tests::step_by_step_without_waiting ... bench:      22,404 ns/iter (+/- 1,980)
test mutex::futures::tests::concurrency_without_waiting                   ... bench:      71,269 ns/iter (+/- 2,695)
test mutex::futures::tests::create                                        ... bench:          93 ns/iter (+/- 9)
test mutex::futures::tests::step_by_step_without_waiting                  ... bench:      22,268 ns/iter (+/- 3,061)
test mutex::smol::tests::concurrency_without_waiting                      ... bench:      65,587 ns/iter (+/- 2,411)
test mutex::smol::tests::create                                           ... bench:           0 ns/iter (+/- 0)
test mutex::smol::tests::step_by_step_without_waiting                     ... bench:      23,112 ns/iter (+/- 2,687)
test mutex::tokio::tests::concurrency_without_waiting                     ... bench:      57,663 ns/iter (+/- 2,737)
test mutex::tokio::tests::create                                          ... bench:           5 ns/iter (+/- 0)
test mutex::tokio::tests::step_by_step_without_waiting                    ... bench:      27,206 ns/iter (+/- 2,714)
test rwlock::fast_async_mutex::tests::concurrency_read                    ... bench:      48,845 ns/iter (+/- 3,166)
test rwlock::fast_async_mutex::tests::concurrency_write                   ... bench:      53,226 ns/iter (+/- 2,136)
test rwlock::fast_async_mutex::tests::create                              ... bench:           0 ns/iter (+/- 0)
test rwlock::fast_async_mutex::tests::step_by_step_read                   ... bench:      22,840 ns/iter (+/- 3,707)
test rwlock::fast_async_mutex::tests::step_by_step_writing                ... bench:      22,274 ns/iter (+/- 2,105)
test rwlock::fast_async_mutex_ordered::tests::concurrency_read            ... bench:      63,874 ns/iter (+/- 2,844)
test rwlock::fast_async_mutex_ordered::tests::concurrency_write           ... bench:      60,157 ns/iter (+/- 10,199)
test rwlock::fast_async_mutex_ordered::tests::create                      ... bench:           0 ns/iter (+/- 0)
test rwlock::fast_async_mutex_ordered::tests::step_by_step_read           ... bench:      23,459 ns/iter (+/- 5,786)
test rwlock::fast_async_mutex_ordered::tests::step_by_step_writing        ... bench:      27,603 ns/iter (+/- 4,806)
test rwlock::smol::tests::concurrency_read                                ... bench:      50,487 ns/iter (+/- 6,896)
test rwlock::smol::tests::concurrency_write                               ... bench:      82,739 ns/iter (+/- 4,317)
test rwlock::smol::tests::create                                          ... bench:           1 ns/iter (+/- 0)
test rwlock::smol::tests::step_by_step_read                               ... bench:      21,993 ns/iter (+/- 2,598)
test rwlock::smol::tests::step_by_step_writing                            ... bench:      25,255 ns/iter (+/- 2,989)
test rwlock::tokio::tests::concurrency_read                               ... bench:      56,158 ns/iter (+/- 1,458)
test rwlock::tokio::tests::concurrency_write                              ... bench:      56,646 ns/iter (+/- 2,227)
test rwlock::tokio::tests::create                                         ... bench:           5 ns/iter (+/- 1)
test rwlock::tokio::tests::step_by_step_read                              ... bench:      26,073 ns/iter (+/- 3,883)
test rwlock::tokio::tests::step_by_step_writing                           ... bench:      26,231 ns/iter (+/- 2,877)

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
