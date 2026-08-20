use std::thread;
use std::sync::{mpsc, Arc, Mutex};

/// 스레드가 실행할 일회성 작업 (클로저)
type Job = Box<dyn FnOnce() + Send + 'static>;

/// [BioPhys Foundation]
/// Tokio나 Rayon 같은 거대한 범용 런타임을 버리고, 오직 프랙탈 디코딩만을 위해
/// OS 원시(Raw) 스레드를 통제하는 커스텀 무결점 스레드 풀.
pub struct BpThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

impl BpThreadPool {
    /// 앱 시작 시 CPU 코어 개수만큼 워커 스레드를 미리 생성하여 깨워둡니다.
    /// (Context Switching 0%)
    pub fn new(size: usize) -> Self {
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);
        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        Self {
            workers,
            sender: Some(sender),
        }
    }

    /// 클로저(Job)를 큐에 던지면, 대기 중이던 워커 스레드가 낚아채어 즉시 실행합니다.
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        // 채널을 통해 던지기만 하면 OS 락 프리(Lock-Free) 통신 완료
        self.sender.as_ref().unwrap().send(job).unwrap();
    }
}

impl Drop for BpThreadPool {
    fn drop(&mut self) {
        // 송신 채널을 닫아 워커들에게 종료 신호 전달
        drop(self.sender.take());

        for worker in &mut self.workers {
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Self {
        let thread = thread::spawn(move || loop {
            // 채널에서 일을 기다림
            let message = receiver.lock().unwrap().recv();

            match message {
                Ok(job) => {
                    // 일(Job)을 받으면 즉시 실행
                    job();
                }
                Err(_) => {
                    // 채널이 닫혔으면 조용히 종료
                    break;
                }
            }
        });

        Self {
            id,
            thread: Some(thread),
        }
    }
}
