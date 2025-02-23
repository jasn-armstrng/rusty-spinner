use std::{
    fmt,
    sync::{mpsc, Arc, Mutex},
    thread,
};

#[derive(Debug)] // Allows println!("{:?}", err); for debugging purposes.
pub enum PoolCreationError {
    InvalidSize,                      // ThreadPool size must be greater than zero
    ThreadSpawnError(std::io::Error), // Failed to spawn thread
}

impl fmt::Display for PoolCreationError {
    // This makes the error message meaningful when printed (println!("{}", err);).
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PoolCreationError::InvalidSize => {
                write!(f, "ThreadPool size must be greater than zero")
            }
            PoolCreationError::ThreadSpawnError(err) => {
                write!(f, "Failed to spawn thread: {}", err)
            }
        }
    }
}

// Implementing std::error::Error allows this custom error to integrate with Rust’s error handling ecosystem
// This makes it compatible with functions that return Box<dyn std::error::Error>
impl std::error::Error for PoolCreationError {}

pub struct Worker {
    id: usize,
    thread: thread::JoinHandle<Arc<Mutex<mpsc::Receiver<Job>>>>,
}

impl Worker {
    // Private because it's an implementation detail of the ThreadPool. Main does not need to know about it.
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Result<Worker, std::io::Error> {
        let builder = thread::Builder::new().name(format!("worker-{}", id));

        // [Question] What does the code below do?
        // It spawns a new thread that will receive jobs from the receiver and execute them.
        let thread = builder.spawn(move || loop {
            let job = receiver
                .lock() // Use to acquire a mutex, blocking the current thread until it is able to do so.
                .expect("Failed to acquire mutex")
                .recv() // Use to receive a job from the receiver
                .unwrap(); // Use to unwrap the received job

            println!("Worker {id} got a job; executing.");

            job();
        })?;

        // If spawn succeeded, return the Worker
        Ok(Worker { id, thread })
    }
}

pub struct ThreadPool {
    workers: Vec<Worker>, // size = 24 (0x18), align = 0x8, offset = 0x10
    sender: mpsc::Sender<Job>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

impl ThreadPool {
    // Constructor for ThreadPool. Creates a new ThreadPool with the given +ve number (usize) of threads.
    pub fn build(size: usize) -> Result<ThreadPool, PoolCreationError> {
        if size == 0 {
            return Err(PoolCreationError::InvalidSize);
        }

        let (sender, receiver) = mpsc::channel(); // [Note] Creates a new asynchronous channel, returning the sender/receiver halves.
        let receiver = Arc::new(Mutex::new(receiver)); // [Note] Arc is a thread-safe reference-counting pointer. ‘Arc’ stands for ‘Atomically Reference Counted’.

        let workers = (0..size)
            .map(|id| Worker::new(id, Arc::clone(&receiver)))
            .collect::<Result<Vec<_>, std::io::Error>>()
            .map_err(PoolCreationError::ThreadSpawnError)?;

        Ok(ThreadPool { workers, sender })
    }

    // Method to execute a closure in the ThreadPool.
    pub fn execute<F>(&self, f: F)
    where
        // [Note] F is a function-like type (a closure) that can be called once.
        // [Note] Use FnOnce as a bound when you want to accept a parameter of function-like (a closure) type and only need to call it once.
        // [Note] Send trait is required to send the closure across threads.
        // [Note] 'static lifetime is required to ensure the closure outlives the thread.
        F: FnOnce() + Send + 'static,
    {
        // Implementation details...
        let job = Box::new(f); // Create a boxed closure
        self.sender.send(job).unwrap(); // Send the job to the receiver
    }
}
