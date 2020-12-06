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
test mutex::fast_async_mutex::tests::concurrency_without_waiting          ... bench:      49,844 ns/iter (+/- 3,223)
test mutex::fast_async_mutex::tests::create                               ... bench:           0 ns/iter (+/- 0)
test mutex::fast_async_mutex::tests::step_by_step_without_waiting         ... bench:      22,685 ns/iter (+/- 18,466)
test mutex::fast_async_mutex_ordered::tests::concurrency_without_waiting  ... bench:      50,173 ns/iter (+/- 1,163)
test mutex::fast_async_mutex_ordered::tests::create                       ... bench:           0 ns/iter (+/- 0)
test mutex::fast_async_mutex_ordered::tests::step_by_step_without_waiting ... bench:      22,007 ns/iter (+/- 2,974)
test mutex::futures::tests::concurrency_without_waiting                   ... bench:      76,535 ns/iter (+/- 12,181)
test mutex::futures::tests::create                                        ... bench:          92 ns/iter (+/- 17)
test mutex::futures::tests::step_by_step_without_waiting                  ... bench:      23,109 ns/iter (+/- 7,627)
test mutex::smol::tests::concurrency_without_waiting                      ... bench:      66,312 ns/iter (+/- 4,924)
test mutex::smol::tests::create                                           ... bench:           0 ns/iter (+/- 0)
test mutex::smol::tests::step_by_step_without_waiting                     ... bench:      28,466 ns/iter (+/- 3,385)
test mutex::tokio::tests::concurrency_without_waiting                     ... bench:      60,833 ns/iter (+/- 11,640)
test mutex::tokio::tests::create                                          ... bench:           6 ns/iter (+/- 3)
test mutex::tokio::tests::step_by_step_without_waiting                    ... bench:      31,924 ns/iter (+/- 9,327)
test rwlock::fast_async_mutex::tests::concurrency_read                    ... bench:      53,613 ns/iter (+/- 4,125)
test rwlock::fast_async_mutex::tests::concurrency_write                   ... bench:      50,652 ns/iter (+/- 1,181)
test rwlock::fast_async_mutex::tests::create                              ... bench:           0 ns/iter (+/- 0)
test rwlock::fast_async_mutex::tests::step_by_step_read                   ... bench:      23,161 ns/iter (+/- 5,225)
test rwlock::fast_async_mutex::tests::step_by_step_writing                ... bench:      23,330 ns/iter (+/- 4,819)
test rwlock::fast_async_mutex_ordered::tests::concurrency_read            ... bench:      50,208 ns/iter (+/- 896)
test rwlock::fast_async_mutex_ordered::tests::concurrency_write           ... bench:      50,227 ns/iter (+/- 1,984)
test rwlock::fast_async_mutex_ordered::tests::create                      ... bench:           0 ns/iter (+/- 0)
test rwlock::fast_async_mutex_ordered::tests::step_by_step_read           ... bench:      23,059 ns/iter (+/- 2,393)
test rwlock::fast_async_mutex_ordered::tests::step_by_step_writing        ... bench:      22,074 ns/iter (+/- 5,107)
test rwlock::smol::tests::concurrency_read                                ... bench:      55,767 ns/iter (+/- 1,843)
test rwlock::smol::tests::concurrency_write                               ... bench:      85,189 ns/iter (+/- 2,852)
test rwlock::smol::tests::create                                          ... bench:           1 ns/iter (+/- 0)
test rwlock::smol::tests::step_by_step_read                               ... bench:      22,644 ns/iter (+/- 1,688)
test rwlock::smol::tests::step_by_step_writing                            ... bench:      25,769 ns/iter (+/- 2,010)
test rwlock::tokio::tests::concurrency_read                               ... bench:      52,960 ns/iter (+/- 939)
test rwlock::tokio::tests::concurrency_write                              ... bench:      60,748 ns/iter (+/- 4,604)
test rwlock::tokio::tests::create                                         ... bench:           5 ns/iter (+/- 1)
test rwlock::tokio::tests::step_by_step_read                              ... bench:      31,437 ns/iter (+/- 6,948)
test rwlock::tokio::tests::step_by_step_writing                           ... bench:      29,766 ns/iter (+/- 8,081)

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
