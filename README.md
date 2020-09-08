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
running 15 tests
test fast_async_mutex::tests::concurrency_without_waiting            ... bench:   1,596,189 ns/iter (+/- 128,344)
test fast_async_mutex::tests::create                                 ... bench:           0 ns/iter (+/- 0)
test fast_async_mutex::tests::step_by_step_without_waiting           ... bench:     172,753 ns/iter (+/- 16,961)
test fast_async_mutex_unordered::tests::concurrency_without_waiting  ... bench:   1,598,340 ns/iter (+/- 124,138)
test fast_async_mutex_unordered::tests::create                       ... bench:           0 ns/iter (+/- 0)
test fast_async_mutex_unordered::tests::step_by_step_without_waiting ... bench:     134,804 ns/iter (+/- 16,646)
test futures::tests::concurrency_without_waiting                     ... bench:   1,665,560 ns/iter (+/- 199,774)
test futures::tests::create                                          ... bench:          89 ns/iter (+/- 24)
test futures::tests::step_by_step_without_waiting                    ... bench:     207,111 ns/iter (+/- 19,629)
test smol::tests::concurrency_without_waiting                        ... bench:   1,881,085 ns/iter (+/- 139,396)
test smol::tests::create                                             ... bench:           0 ns/iter (+/- 0)
test smol::tests::step_by_step_without_waiting                       ... bench:     342,648 ns/iter (+/- 74,680)
test tokio::tests::concurrency_without_waiting                       ... bench:  22,845,551 ns/iter (+/- 1,920,176)
test tokio::tests::create                                            ... bench:          88 ns/iter (+/- 22)
test tokio::tests::step_by_step_without_waiting                      ... bench:     723,938 ns/iter (+/- 131,765)

test result: ok. 0 passed; 0 failed; 0 ignored; 15 measured; 0 filtered out
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
