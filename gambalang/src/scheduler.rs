use std::collections::VecDeque;
use std::sync::{Arc, Condvar, Mutex, OnceLock};
use std::thread::{self, JoinHandle};

type Job = Box<dyn FnOnce() + Send + 'static>;

struct Queue {
    inner: Mutex<Inner>,
    cv: Condvar,
}

struct Inner {
    q: VecDeque<Job>,
    shutdown: bool,
}

pub struct ThreadPool {
    workers: Vec<JoinHandle<()>>,
    queue: Arc<Queue>,
}

impl ThreadPool {
    pub fn new(size: usize) -> Self {
        let queue = Arc::new(Queue {
            inner: Mutex::new(Inner {
                q: VecDeque::new(),
                shutdown: false,
            }),
            cv: Condvar::new(),
        });

        let mut workers = Vec::with_capacity(size);
        for _ in 0..size {
            let q = queue.clone();
            workers.push(thread::spawn(move || worker_loop(q)));
        }
        ThreadPool { workers, queue }
    }

    pub fn spawn(&self, job: Job) {
        let mut guard = self.queue.inner.lock().unwrap();
        guard.q.push_back(job);
        self.queue.cv.notify_one();
    }

    fn shutdown(&self) {
        let mut guard = self.queue.inner.lock().unwrap();
        guard.shutdown = true;
        self.queue.cv.notify_all();
    }
}

fn worker_loop(q: Arc<Queue>) {
    loop {
        let job = {
            let mut guard = q.inner.lock().unwrap();
            loop {
                if let Some(j) = guard.q.pop_front() {
                    break j;
                }
                if guard.shutdown {
                    return;
                }
                guard = q.cv.wait(guard).unwrap();
            }
        };
        job();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        self.shutdown();
        for w in self.workers.drain(..) {
            let _ = w.join();
        }
    }
}

static GLOBAL_POOL: OnceLock<ThreadPool> = OnceLock::new();

pub fn pool() -> &'static ThreadPool {
    GLOBAL_POOL.get_or_init(|| {
        let n = std::thread::available_parallelism()
            .map(|x| x.get())
            .unwrap_or(4);
        ThreadPool::new(n.max(2)) // pelo menos 2
    })
}
