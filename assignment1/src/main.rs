use std::sync::mpsc::channel;
use uuid::Uuid;
use std::thread;
use rand::Rng;
use std::time::Duration;
const NTHREADS: u32 = 100;
const TIMEOUT_MILIS: u64 = 5000;
use bus::Bus;

struct Task {
    id: u32,
    payload: String,
}

impl Task {
    fn create_task(id: u32, payload: &str) -> Task {
        Task {id: id, payload: payload.to_string()}
    }
}

struct Worker {
    id: u32,
}

impl Worker {
    fn process_task(&self, task: &Task) -> String {
        format!("worker id {} processed task with id {} and payload {}", self.id, task.id, task.payload)
    }
}

fn main() {
    let (tx, rx) = channel();
    let mut bus = Bus::new(NTHREADS as usize);
    let mut tasks = vec![];
    let mut join_handles = vec![];
    for i in 0..NTHREADS {
        tasks.push(Task::create_task(i, &Uuid::new_v4().to_string()));
    }
    for (i, task) in tasks.into_iter().enumerate()  {
        let tx = tx.clone();
        let mut rx_timeout = bus.add_rx();
        join_handles.push(thread::spawn(move || {
            let num = rand::thread_rng().gen_range(0..10000);
            let worker = Worker{id: i as u32};
            if let Ok(_) = rx_timeout.recv_timeout(Duration::from_millis (num)) {
                    // Sleep was interrupted
                    tx.send(format!("worker id {} timed out", worker.id)).unwrap();
                    return
            }
            tx.send(worker.process_task(&task)).unwrap();
        }));
    }
    drop(tx);

    //timer thread will broadcast timeout if more than TIMEOUT_MILIS elapse, but the main thread can exit earlier if it's done
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(TIMEOUT_MILIS));
        bus.broadcast(1);
    });

    for handle in join_handles.into_iter() {
        handle.join().unwrap();
        for received in &rx {
            println!("Received: {}", received);
        }
    }   
}
