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
test mutex::fast_async_mutex::tests::concurrency_without_waiting          ... bench:   1,655,316 ns/iter (+/- 130,153)
test mutex::fast_async_mutex::tests::create                               ... bench:           0 ns/iter (+/- 0)
test mutex::fast_async_mutex::tests::step_by_step_without_waiting         ... bench:     209,525 ns/iter (+/- 41,279)
test mutex::fast_async_mutex_ordered::tests::concurrency_without_waiting  ... bench:   1,658,499 ns/iter (+/- 227,813)
test mutex::fast_async_mutex_ordered::tests::create                       ... bench:           0 ns/iter (+/- 0)
test mutex::fast_async_mutex_ordered::tests::step_by_step_without_waiting ... bench:     228,778 ns/iter (+/- 57,589)
test mutex::futures::tests::concurrency_without_waiting                   ... bench:   1,717,780 ns/iter (+/- 339,007)
test mutex::futures::tests::create                                        ... bench:          89 ns/iter (+/- 20)
test mutex::futures::tests::step_by_step_without_waiting                  ... bench:     208,992 ns/iter (+/- 39,132)
test mutex::smol::tests::concurrency_without_waiting                      ... bench:   1,898,404 ns/iter (+/- 205,293)
test mutex::smol::tests::create                                           ... bench:           0 ns/iter (+/- 0)
test mutex::smol::tests::step_by_step_without_waiting                     ... bench:     337,008 ns/iter (+/- 42,071)
test mutex::tokio::tests::concurrency_without_waiting                     ... bench:  22,084,099 ns/iter (+/- 3,158,231)
test mutex::tokio::tests::create                                          ... bench:          87 ns/iter (+/- 19)
test mutex::tokio::tests::step_by_step_without_waiting                    ... bench:     700,349 ns/iter (+/- 104,633)
test rwlock::fast_async_mutex::tests::concurrency_read                    ... bench:   1,706,870 ns/iter (+/- 162,620)
test rwlock::fast_async_mutex::tests::concurrency_write                   ... bench:   1,580,081 ns/iter (+/- 277,137)
test rwlock::fast_async_mutex::tests::create                              ... bench:           0 ns/iter (+/- 0)
test rwlock::fast_async_mutex::tests::step_by_step_read                   ... bench:     249,690 ns/iter (+/- 44,143)
test rwlock::fast_async_mutex::tests::step_by_step_writing                ... bench:     173,125 ns/iter (+/- 31,013)
test rwlock::fast_async_mutex_ordered::tests::concurrency_read            ... bench:   1,780,221 ns/iter (+/- 135,192)
test rwlock::fast_async_mutex_ordered::tests::concurrency_write           ... bench:   1,654,904 ns/iter (+/- 147,652)
test rwlock::fast_async_mutex_ordered::tests::create                      ... bench:           0 ns/iter (+/- 0)
test rwlock::fast_async_mutex_ordered::tests::step_by_step_read           ... bench:     323,291 ns/iter (+/- 63,657)
test rwlock::fast_async_mutex_ordered::tests::step_by_step_writing        ... bench:     212,939 ns/iter (+/- 40,587)
test rwlock::smol::tests::concurrency_read                                ... bench:   1,686,007 ns/iter (+/- 448,853)
test rwlock::smol::tests::concurrency_write                               ... bench:   2,115,666 ns/iter (+/- 178,394)
test rwlock::smol::tests::create                                          ... bench:           1 ns/iter (+/- 0)
test rwlock::smol::tests::step_by_step_read                               ... bench:     189,378 ns/iter (+/- 38,296)
test rwlock::smol::tests::step_by_step_writing                            ... bench:     592,654 ns/iter (+/- 149,131)
test rwlock::tokio::tests::concurrency_read                               ... bench:  23,426,186 ns/iter (+/- 2,685,113)
test rwlock::tokio::tests::concurrency_write                              ... bench:  23,410,050 ns/iter (+/- 2,571,675)
test rwlock::tokio::tests::create                                         ... bench:          87 ns/iter (+/- 17)
test rwlock::tokio::tests::step_by_step_read                              ... bench:     850,208 ns/iter (+/- 237,666)
test rwlock::tokio::tests::step_by_step_writing                           ... bench:     805,490 ns/iter (+/- 115,358)

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
