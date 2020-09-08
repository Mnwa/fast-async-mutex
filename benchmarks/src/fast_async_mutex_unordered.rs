#[cfg(test)]
mod tests {
    use fast_async_mutex::mutex_unordered::{Mutex, MutexGuard};
    use futures::StreamExt;
    use test::Bencher;

    #[bench]
    fn create(b: &mut Bencher) {
        b.iter(|| Mutex::new(()));
    }

    #[bench]
    fn concurrency_without_waiting(b: &mut Bencher) {
        let mut runtime = tokio::runtime::Builder::new()
            .enable_all()
            .threaded_scheduler()
            .build()
            .unwrap();
        b.iter(|| {
            runtime.block_on(async {
                let c = Mutex::new(0);

                futures::stream::iter(0..10000u64)
                    .for_each_concurrent(None, |_| async {
                        let mut co: MutexGuard<i32> = c.lock().await;
                        *co += 1;
                    })
                    .await;
            })
        });
    }

    #[bench]
    fn step_by_step_without_waiting(b: &mut Bencher) {
        let mut runtime = tokio::runtime::Builder::new()
            .enable_all()
            .threaded_scheduler()
            .build()
            .unwrap();
        b.iter(|| {
            runtime.block_on(async {
                let c = Mutex::new(0);

                futures::stream::iter(0..10000i32)
                    .for_each(|_| async {
                        let mut co: MutexGuard<i32> = c.lock().await;
                        *co += 1;
                    })
                    .await;
            })
        });
    }
}
