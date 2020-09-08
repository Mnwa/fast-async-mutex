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
running 30 tests
test mutex::fast_async_mutex::tests::concurrency_without_waiting            ... bench:   1,582,255 ns/iter (+/- 226,548)
test mutex::fast_async_mutex::tests::create                                 ... bench:           0 ns/iter (+/- 0)
test mutex::fast_async_mutex::tests::step_by_step_without_waiting           ... bench:     165,499 ns/iter (+/- 58,886)
test mutex::fast_async_mutex_unordered::tests::concurrency_without_waiting  ... bench:   1,559,522 ns/iter (+/- 213,015)
test mutex::fast_async_mutex_unordered::tests::create                       ... bench:           0 ns/iter (+/- 0)
test mutex::fast_async_mutex_unordered::tests::step_by_step_without_waiting ... bench:      92,905 ns/iter (+/- 13,518)
test mutex::futures::tests::concurrency_without_waiting                     ... bench:   1,636,030 ns/iter (+/- 109,052)
test mutex::futures::tests::create                                          ... bench:          88 ns/iter (+/- 15)
test mutex::futures::tests::step_by_step_without_waiting                    ... bench:     210,283 ns/iter (+/- 15,931)
test mutex::smol::tests::concurrency_without_waiting                        ... bench:   1,870,199 ns/iter (+/- 142,146)
test mutex::smol::tests::create                                             ... bench:           0 ns/iter (+/- 0)
test mutex::smol::tests::step_by_step_without_waiting                       ... bench:     343,573 ns/iter (+/- 32,607)
test mutex::tokio::tests::concurrency_without_waiting                       ... bench:  22,686,028 ns/iter (+/- 3,648,661)
test mutex::tokio::tests::create                                            ... bench:          89 ns/iter (+/- 10)
test mutex::tokio::tests::step_by_step_without_waiting                      ... bench:     707,779 ns/iter (+/- 114,661)
test rwlock::fast_async_mutex::tests::concurrency_read                      ... bench:   1,698,161 ns/iter (+/- 153,215)
test rwlock::fast_async_mutex::tests::concurrency_write                     ... bench:   1,584,764 ns/iter (+/- 115,173)
test rwlock::fast_async_mutex::tests::create                                ... bench:           0 ns/iter (+/- 0)
test rwlock::fast_async_mutex::tests::step_by_step_read                     ... bench:     247,954 ns/iter (+/- 33,076)
test rwlock::fast_async_mutex::tests::step_by_step_writing                  ... bench:     153,154 ns/iter (+/- 27,294)
test rwlock::smol::tests::concurrency_read                                  ... bench:   1,667,601 ns/iter (+/- 188,827)
test rwlock::smol::tests::concurrency_write                                 ... bench:   2,139,812 ns/iter (+/- 208,289)
test rwlock::smol::tests::create                                            ... bench:           1 ns/iter (+/- 0)
test rwlock::smol::tests::step_by_step_read                                 ... bench:     196,376 ns/iter (+/- 26,671)
test rwlock::smol::tests::step_by_step_writing                              ... bench:     623,069 ns/iter (+/- 121,692)
test rwlock::tokio::tests::concurrency_read                                 ... bench:  24,207,303 ns/iter (+/- 3,060,491)
test rwlock::tokio::tests::concurrency_write                                ... bench:  23,943,930 ns/iter (+/- 1,776,313)
test rwlock::tokio::tests::create                                           ... bench:          86 ns/iter (+/- 19)
test rwlock::tokio::tests::step_by_step_read                                ... bench:     815,935 ns/iter (+/- 58,694)
test rwlock::tokio::tests::step_by_step_writing                             ... bench:     841,960 ns/iter (+/- 180,398)

test result: ok. 0 passed; 0 failed; 0 ignored; 30 measured; 0 filtered out
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
