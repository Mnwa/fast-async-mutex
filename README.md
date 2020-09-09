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
test mutex::fast_async_mutex::tests::concurrency_without_waiting            ... bench:   1,670,490 ns/iter (+/- 248,420)
test mutex::fast_async_mutex::tests::create                                 ... bench:           0 ns/iter (+/- 0)
test mutex::fast_async_mutex::tests::step_by_step_without_waiting           ... bench:     227,009 ns/iter (+/- 21,953)
test mutex::fast_async_mutex_unordered::tests::concurrency_without_waiting  ... bench:   1,651,841 ns/iter (+/- 157,781)
test mutex::fast_async_mutex_unordered::tests::create                       ... bench:           0 ns/iter (+/- 0)
test mutex::fast_async_mutex_unordered::tests::step_by_step_without_waiting ... bench:     198,434 ns/iter (+/- 60,530)
test mutex::futures::tests::concurrency_without_waiting                     ... bench:   1,701,189 ns/iter (+/- 164,287)
test mutex::futures::tests::create                                          ... bench:          89 ns/iter (+/- 24)
test mutex::futures::tests::step_by_step_without_waiting                    ... bench:     206,938 ns/iter (+/- 39,139)
test mutex::smol::tests::concurrency_without_waiting                        ... bench:   1,839,102 ns/iter (+/- 286,252)
test mutex::smol::tests::create                                             ... bench:           0 ns/iter (+/- 0)
test mutex::smol::tests::step_by_step_without_waiting                       ... bench:     342,488 ns/iter (+/- 98,159)
test mutex::tokio::tests::concurrency_without_waiting                       ... bench:  22,299,201 ns/iter (+/- 3,475,108)
test mutex::tokio::tests::create                                            ... bench:          86 ns/iter (+/- 32)
test mutex::tokio::tests::step_by_step_without_waiting                      ... bench:     727,885 ns/iter (+/- 109,948)
test rwlock::fast_async_mutex::tests::concurrency_read                      ... bench:   1,842,199 ns/iter (+/- 266,238)
test rwlock::fast_async_mutex::tests::concurrency_write                     ... bench:   1,659,933 ns/iter (+/- 203,743)
test rwlock::fast_async_mutex::tests::create                                ... bench:           0 ns/iter (+/- 0)
test rwlock::fast_async_mutex::tests::step_by_step_read                     ... bench:     308,579 ns/iter (+/- 40,304)
test rwlock::fast_async_mutex::tests::step_by_step_writing                  ... bench:     210,990 ns/iter (+/- 29,719)
test rwlock::smol::tests::concurrency_read                                  ... bench:   1,670,629 ns/iter (+/- 172,639)
test rwlock::smol::tests::concurrency_write                                 ... bench:   2,092,212 ns/iter (+/- 322,091)
test rwlock::smol::tests::create                                            ... bench:           1 ns/iter (+/- 0)
test rwlock::smol::tests::step_by_step_read                                 ... bench:     192,174 ns/iter (+/- 35,883)
test rwlock::smol::tests::step_by_step_writing                              ... bench:     587,333 ns/iter (+/- 72,670)
test rwlock::tokio::tests::concurrency_read                                 ... bench:  23,133,804 ns/iter (+/- 2,175,812)
test rwlock::tokio::tests::concurrency_write                                ... bench:  23,657,362 ns/iter (+/- 4,149,170)
test rwlock::tokio::tests::create                                           ... bench:          82 ns/iter (+/- 15)
test rwlock::tokio::tests::step_by_step_read                                ... bench:     791,857 ns/iter (+/- 95,246)
test rwlock::tokio::tests::step_by_step_writing                             ... bench:     797,907 ns/iter (+/- 213,888)

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
