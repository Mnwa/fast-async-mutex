# fast-async-mutex
[![](https://github.com/Mnwa/fast-async-mutex/workflows/build/badge.svg?branch=master)](https://github.com/Mnwa/fast-async-mutex/actions?query=workflow%3Abuild)
[![](https://docs.rs/fast-async-mutex/badge.svg)](https://docs.rs/fast-async-mutex/)
[![](https://img.shields.io/crates/v/fast-async-mutex.svg)](https://crates.io/crates/fast-async-mutex)
[![](https://img.shields.io/crates/d/fast-async-mutex.svg)](https://crates.io/crates/fast-async-mutex)

The fast async mutex which uses atomics with spinlock algorithm. 
Spinlock algorithm integrated with the Rust futures concept without the overhead of any libs. Also when the `MutexGuard` is dropped,
a waker of the next locker will be executed.
It will be works with any async runtime in `Rust`, it may be a `tokio`, `smol`, `async-std` and etc..


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
test mutex::fast_async_mutex::tests::concurrency_without_waiting            ... bench:   1,708,681 ns/iter (+/- 281,626)
test mutex::fast_async_mutex::tests::create                                 ... bench:           0 ns/iter (+/- 0)
test mutex::fast_async_mutex::tests::step_by_step_without_waiting           ... bench:     176,524 ns/iter (+/- 25,409)
test mutex::fast_async_mutex_unordered::tests::concurrency_without_waiting  ... bench:   1,623,922 ns/iter (+/- 175,048)
test mutex::fast_async_mutex_unordered::tests::create                       ... bench:           0 ns/iter (+/- 0)
test mutex::fast_async_mutex_unordered::tests::step_by_step_without_waiting ... bench:      96,135 ns/iter (+/- 18,169)
test mutex::futures::tests::concurrency_without_waiting                     ... bench:   1,708,160 ns/iter (+/- 272,253)
test mutex::futures::tests::create                                          ... bench:          90 ns/iter (+/- 22)
test mutex::futures::tests::step_by_step_without_waiting                    ... bench:     239,459 ns/iter (+/- 140,015)
test mutex::smol::tests::concurrency_without_waiting                        ... bench:   2,171,628 ns/iter (+/- 539,797)
test mutex::smol::tests::create                                             ... bench:           0 ns/iter (+/- 0)
test mutex::smol::tests::step_by_step_without_waiting                       ... bench:     347,041 ns/iter (+/- 98,376)
test mutex::tokio::tests::concurrency_without_waiting                       ... bench:  23,091,726 ns/iter (+/- 2,934,483)
test mutex::tokio::tests::create                                            ... bench:          88 ns/iter (+/- 48)
test mutex::tokio::tests::step_by_step_without_waiting                      ... bench:     731,412 ns/iter (+/- 121,916)
test rwlock::fast_async_mutex::tests::concurrency_read                      ... bench:   1,754,822 ns/iter (+/- 201,042)
test rwlock::fast_async_mutex::tests::concurrency_write                     ... bench:   1,586,216 ns/iter (+/- 214,710)
test rwlock::fast_async_mutex::tests::create                                ... bench:           0 ns/iter (+/- 0)
test rwlock::fast_async_mutex::tests::step_by_step_read                     ... bench:     265,539 ns/iter (+/- 100,065)
test rwlock::fast_async_mutex::tests::step_by_step_writing                  ... bench:     155,619 ns/iter (+/- 27,964)
test rwlock::smol::tests::concurrency_read                                  ... bench:   1,695,213 ns/iter (+/- 188,374)
test rwlock::smol::tests::concurrency_write                                 ... bench:   2,160,869 ns/iter (+/- 186,303)
test rwlock::smol::tests::create                                            ... bench:           1 ns/iter (+/- 0)
test rwlock::smol::tests::step_by_step_read                                 ... bench:     214,860 ns/iter (+/- 71,858)
test rwlock::smol::tests::step_by_step_writing                              ... bench:     600,193 ns/iter (+/- 45,711)
test rwlock::tokio::tests::concurrency_read                                 ... bench:  23,941,650 ns/iter (+/- 2,390,315)
test rwlock::tokio::tests::concurrency_write                                ... bench:  23,464,154 ns/iter (+/- 1,475,203)
test rwlock::tokio::tests::create                                           ... bench:          90 ns/iter (+/- 27)
test rwlock::tokio::tests::step_by_step_read                                ... bench:     816,434 ns/iter (+/- 124,253)
test rwlock::tokio::tests::step_by_step_writing                             ... bench:     822,166 ns/iter (+/- 134,409)

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
