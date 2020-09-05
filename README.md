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
running 12 tests
test fast_async_mutex::tests::concurrency_without_waiting  ... bench:   1,579,327 ns/iter (+/- 122,787)
test fast_async_mutex::tests::create                       ... bench:           0 ns/iter (+/- 0)
test fast_async_mutex::tests::step_by_step_without_waiting ... bench:     194,037 ns/iter (+/- 18,457)
test futures::tests::concurrency_without_waiting           ... bench:   1,623,903 ns/iter (+/- 198,099)
test futures::tests::create                                ... bench:          84 ns/iter (+/- 20)
test futures::tests::step_by_step_without_waiting          ... bench:     205,545 ns/iter (+/- 11,437)
test smol::tests::concurrency_without_waiting              ... bench:   1,813,222 ns/iter (+/- 88,291)
test smol::tests::create                                   ... bench:           0 ns/iter (+/- 0)
test smol::tests::step_by_step_without_waiting             ... bench:     329,124 ns/iter (+/- 33,841)
test tokio::tests::concurrency_without_waiting             ... bench:  22,625,869 ns/iter (+/- 6,185,517)
test tokio::tests::create                                  ... bench:          98 ns/iter (+/- 54)
test tokio::tests::step_by_step_without_waiting            ... bench:     718,106 ns/iter (+/- 157,179)

test result: ok. 0 passed; 0 failed; 0 ignored; 12 measured; 0 filtered out
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
