//type alias
type Job = Box<dyn FnOnce() + Send + 'static>;

use std::{
    sync::{
        mpsc::{self, Receiver, Sender},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
};

pub struct ThreadPool {
    workers: Vec<Worker>,
    tx: Option<Sender<Job>>,
}

pub struct Worker {
    id: usize,
    thread: Option<JoinHandle<()>>,
}

// struct Job {
//     // f: Box<dyn FnOnce()>,
// }

impl Worker {
    fn new(id: usize, rx: Arc<Mutex<Receiver<Job>>>) -> Self {
        let thread = thread::spawn(move || {
            //call to recv blocks the current thread
            // while let Ok(job) = rx.lock().unwrap().recv() {//this line won't work
            loop {
                //continuously asking for job
                let job = rx.lock().unwrap().recv();

                match job {
                    Ok(job) => {
                        //execute the job
                        println!("Worker {id} got a job; executing...");
                        job();
                        println!("Worker {id} finished execution");
                    }
                    Err(err) => {
                        println!("Error: {err}. Worker {id} disconnecting...");
                        break;
                    }
                }
            }
            // if let Ok(job) = rx.lock().unwrap().recv() {
            //     //execute the job
            //     println!("Worker {id} got a job; executing...");
            //     job();
            // }
        });
        Worker {
            id,
            thread: Some(thread),
        }
    }
}

impl ThreadPool {
    /// Create a new ThreadPool.
    ///
    /// The size is the number of threads in the pool.
    ///
    /// # Panics
    ///
    /// The `new` function will panic if the size is zero.
    pub fn new(num: usize) -> Self {
        assert!(num > 0, "Number of threads needs to be greater than zero!");

        //create channel
        let (tx, rx) = mpsc::channel();

        let rx = Arc::new(Mutex::new(rx));

        let mut workers = Vec::with_capacity(num); //set capacity
        for i in 0..num {
            // let rx = rx.clone();
            workers.push(Worker::new(i, rx.clone()));
        }

        ThreadPool {
            workers,
            tx: Some(tx),
        }
    }

    // pub fn build(num: usize) -> Result<Self,PoolCreationError> {

    //     ThreadPool {}
    // }

    // pub fn execute<F>(&self, f: Job) {
    //     // self.tx.send(Job { f });
    //     self.tx.send(f).unwrap();
    // }
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        // self.tx.send(Job { f });
        let job = Box::new(f);

        self.tx.as_ref().unwrap().send(job).unwrap();

        // if let Some(sender) = &self.tx {
        //     sender.send(Box::new(f)).unwrap();
        // }
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        //drop sender
        if let Some(sender) = self.tx.take() {
            drop(sender);
        }

        println!("total workers: {}", self.workers.len());

        for worker in &mut self.workers {
            //let thread = worker.thread.take();

            //take thread out of worker and join
            if let Some(thread) = worker.thread.take() {
                //due to the thread's infinite loop, need to drop sender first, then let the threads finish
                println!("Finishing execution on worker {}", worker.id);
                thread.join().unwrap();
                println!("Shutting down worker {}", worker.id);
            }
        }
    }
}
