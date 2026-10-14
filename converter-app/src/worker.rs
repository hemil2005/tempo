use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc, Condvar, Mutex};
use std::thread;

use crate::job::JobTask;

#[derive(Debug, Clone)]
pub enum WorkerEvent {
    Progress(u64, f32),
    Finished(u64, Result<(), String>),
}

pub struct WorkerPool {
    pub cancel: Arc<AtomicBool>,
    pub pause: Arc<AtomicBool>,
    pause_cond: Arc<(Mutex<bool>, Condvar)>,
    _handles: Vec<thread::JoinHandle<()>>,
}

impl WorkerPool {
    pub fn pause(&self) {
        self.pause.store(true, Ordering::SeqCst);
        let mut lock = self.pause_cond.0.lock().expect("pause lock");
        *lock = true;
    }

    pub fn resume(&self) {
        self.pause.store(false, Ordering::SeqCst);
        let mut lock = self.pause_cond.0.lock().expect("pause lock");
        *lock = false;
        self.pause_cond.1.notify_all();
    }

    pub fn cancel(&self) {
        self.cancel.store(true, Ordering::SeqCst);
        let mut lock = self.pause_cond.0.lock().expect("pause lock");
        *lock = false;
        self.pause_cond.1.notify_all();
    }
}

pub fn spawn_workers(
    tasks: Vec<JobTask>,
    tx: mpsc::Sender<WorkerEvent>,
    cancel: Arc<AtomicBool>,
    pause: Arc<AtomicBool>,
    pause_cond: Arc<(Mutex<bool>, Condvar)>,
    num_workers: usize,
) -> WorkerPool {
    let queue = Arc::new(Mutex::new(VecDeque::from(tasks)));
    let mut handles = Vec::new();

    for _ in 0..num_workers.max(1) {
        let queue = queue.clone();
        let tx = tx.clone();
        let cancel = cancel.clone();
        let pause_cond = pause_cond.clone();
        handles.push(thread::spawn(move || loop {
            {
                let mut paused = pause_cond.0.lock().expect("pause lock");
                while *paused && !cancel.load(Ordering::SeqCst) {
                    paused = pause_cond.1.wait(paused).expect("condvar wait");
                }
            }
            if cancel.load(Ordering::SeqCst) {
                let task: Option<JobTask> = queue.lock().expect("queue lock").pop_front();
                let Some(task) = task else { break };
                let _ = tx.send(WorkerEvent::Finished(task.id, Err("cancelled".to_string())));
                continue;
            }
            let task: Option<JobTask> = queue.lock().expect("queue lock").pop_front();
            let Some(task) = task else { break };
            if cancel.load(Ordering::SeqCst) {
                let _ = tx.send(WorkerEvent::Finished(task.id, Err("cancelled".to_string())));
                continue;
            }
            let id = task.id;
            let result = std::panic::catch_unwind(|| convert_task(&task, &tx, id))
                .unwrap_or_else(|_| Err("worker crashed".to_string()));
            let _ = tx.send(WorkerEvent::Finished(id, result));
        }));
    }

    WorkerPool {
        cancel,
        pause,
        pause_cond,
        _handles: handles,
    }
}

fn convert_task(task: &JobTask, tx: &mpsc::Sender<WorkerEvent>, id: u64) -> Result<(), String> {
    let decoded = converter_core::decode_file(&task.input).map_err(|e| format!("decode: {}", e))?;
    if let Some(parent) = task.output.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("mkdir: {}", e))?;
    }
    let tx = tx.clone();
    converter_core::convert(&decoded, &task.settings, &task.output, &move |p: f32| {
        let _ = tx.send(WorkerEvent::Progress(id, p));
    })
    .map_err(|e| format!("encode: {}", e))?;
    Ok(())
}

pub fn suggested_worker_count() -> usize {
    std::thread::available_parallelism()
        .map(|n| (n.get() / 2).clamp(1, 8))
        .unwrap_or(2)
}
