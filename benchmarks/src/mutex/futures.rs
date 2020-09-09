#[cfg(test)]
mod tests {
    use futures::executor::block_on;
    use futures::lock::Mutex;
    use std::sync::Arc;
    use test::Bencher;

    #[bench]
    fn create(b: &mut Bencher) {
        b.iter(|| Mutex::new(()));
    }

    #[bench]
    fn concurrency_without_waiting(b: &mut Bencher) {
        b.iter(|| {
            let num = 100;
            let mutex = Arc::new(Mutex::new(0));

            let ths: Vec<_> = (0..num)
                .map(|_| {
                    let mutex = mutex.clone();
                    std::thread::spawn(move || {
                        block_on(async {
                            let mut lock = mutex.lock().await;
                            *lock += 1;
                        })
                    })
                })
                .collect();

            for thread in ths {
                thread.join().unwrap();
            }
        })
    }

    #[bench]
    fn step_by_step_without_waiting(b: &mut Bencher) {
        b.iter(|| {
            let num = 100;
            let mutex = Mutex::new(0);

            for _ in 0..num {
                block_on(async {
                    let mut lock = mutex.lock().await;
                    *lock += 1;
                })
            }
        })
    }
}
