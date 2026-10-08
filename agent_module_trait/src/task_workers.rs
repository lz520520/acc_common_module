use std::sync::Mutex;
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

struct WorkerState {
    accepting: bool,
    handles: Vec<JoinHandle<()>>,
}

/// Tracks every thread owned by one task, including threads started by another worker.
pub struct TaskWorkers {
    state: Mutex<WorkerState>,
}

impl Default for TaskWorkers {
    fn default() -> Self {
        Self {
            state: Mutex::new(WorkerState {
                accepting: true,
                handles: Vec::new(),
            }),
        }
    }
}

impl TaskWorkers {
    pub fn spawn<F>(&self, work: F) -> bool
    where
        F: FnOnce() + Send + 'static,
    {
        let mut state = self.state.lock().unwrap();
        if !state.accepting {
            return false;
        }
        state.handles.push(thread::spawn(work));
        true
    }

    pub fn stop_accepting(&self) {
        self.state.lock().unwrap().accepting = false;
    }

    pub fn wait_stopped(&self, deadline: Instant) -> bool {
        loop {
            let (finished, remaining) = {
                let mut state = self.state.lock().unwrap();
                let mut finished = Vec::new();
                let mut index = 0;
                while index < state.handles.len() {
                    if state.handles[index].is_finished() {
                        finished.push(state.handles.swap_remove(index));
                    } else {
                        index += 1;
                    }
                }
                (finished, state.handles.len())
            };
            for handle in finished {
                let _ = handle.join();
            }
            if remaining == 0 {
                return true;
            }
            if Instant::now() >= deadline {
                return false;
            }
            thread::sleep(Duration::from_millis(10));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Barrier};

    #[test]
    fn stop_waits_for_worker_and_rejects_new_work() {
        let workers = TaskWorkers::default();
        let gate = Arc::new(Barrier::new(2));
        let worker_gate = gate.clone();
        assert!(workers.spawn(move || {
            worker_gate.wait();
        }));
        workers.stop_accepting();
        assert!(!workers.spawn(|| {}));
        assert!(!workers.wait_stopped(Instant::now() + Duration::from_millis(20)));
        gate.wait();
        assert!(workers.wait_stopped(Instant::now() + Duration::from_secs(1)));
    }
}
