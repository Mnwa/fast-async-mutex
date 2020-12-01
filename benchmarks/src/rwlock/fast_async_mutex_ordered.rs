#[cfg(test)]
mod tests {
    use fast_async_mutex::rwlock_ordered::OrderedRwLock;
    use std::sync::Arc;
    use test::Bencher;

    #[bench]
    fn create(b: &mut Bencher) {
        b.iter(|| OrderedRwLock::new(()));
    }

    #[bench]
    fn concurrency_write(b: &mut Bencher) {
        let runtime = tokio::runtime::Builder::new_multi_thread().build().unwrap();
        b.iter(|| {
            let num = 100;
            let mutex = Arc::new(OrderedRwLock::new(0));
            let ths: Vec<_> = (0..num)
                .map(|_| {
                    let mutex = mutex.clone();
                    runtime.spawn(async move {
                        let mut lock = mutex.write().await;
                        *lock += 1;
                    })
                })
                .collect();

            for thread in ths {
                runtime.block_on(thread).unwrap();
            }
        });
    }

    #[bench]
    fn step_by_step_writing(b: &mut Bencher) {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap();
        b.iter(|| {
            let num = 100;
            let mutex = OrderedRwLock::new(0);
            for _ in 0..num {
                runtime.block_on(async {
                    let mut lock = mutex.write().await;
                    *lock += 1;
                })
            }
        });
    }

    #[bench]
    fn concurrency_read(b: &mut Bencher) {
        let runtime = tokio::runtime::Builder::new_multi_thread().build().unwrap();
        b.iter(|| {
            let num = 100;
            let mutex = Arc::new(OrderedRwLock::new(0));
            let ths: Vec<_> = (0..num)
                .map(|_| {
                    let mutex = mutex.clone();
                    runtime.spawn(async move {
                        let _lock = mutex.read().await;
                    })
                })
                .collect();

            for thread in ths {
                runtime.block_on(thread).unwrap();
            }
        });
    }

    #[bench]
    fn step_by_step_read(b: &mut Bencher) {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap();
        b.iter(|| {
            let num = 100;
            let mutex = OrderedRwLock::new(0);
            for _ in 0..num {
                runtime.block_on(async {
                    let _lock = mutex.read().await;
                })
            }
        });
    }
}
