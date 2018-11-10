use std::process::{id, Child, Command, Stdio};
use std::sync::{Arc, Mutex};

pub struct ProcessPool {
    processes: Arc<Mutex<Vec<Child>>>,
    thread_handle: Option<std::thread::JoinHandle<()>>,
    tx: std::sync::mpsc::Sender<()>,
}

impl Drop for ProcessPool {
    fn drop(&mut self) {
        self.tx.send(()).unwrap();
        self.thread_handle
            .take()
            .expect("This join handle was alread `take'd` somehow.")
            .join()
            .expect("Couldn't load process pool thread.");
    }
}


impl ProcessPool {
    pub fn new() -> Self {
        let processes = Arc::new(Mutex::new(vec![]));
        let (tx, rx) = std::sync::mpsc::channel();
        let cloned = processes.clone();
        ProcessPool {
            processes: processes,
            thread_handle: Some(std::thread::spawn(move || loop {
                let mut processes = cloned.lock().unwrap();

                processes.drain_filter(|process| {
                    let finished = process.try_wait();

                    match finished {
                        Ok(Some(status)) => {
                            println!("Command exited with code: {}", status.code().unwrap());
                            true
                        }
                        Ok(None) => false,
                        //The process is
                        _ => false,
                    }
                });

                if let Ok(_) = rx.try_recv() {
                    break;
                }
            })),
            tx: tx,
        }
    }

    pub fn add(&mut self, command: &str, mut args: Vec<&str>) {
        let backgrounded = if let Some(arg) = args.last() {
            if *arg == "&" {
                args.pop();
                true
            } else {
                false
            }
        } else {
            false
        };

        let command = if backgrounded {
            Some(
                Command::new(command)
                    .args(args)
                    .stdin(Stdio::null())
                    .spawn()
                    .unwrap(),
            )
        } else {
            match Command::new(command)
                .args(args)
                .stdin(Stdio::null())
                .spawn()
                .unwrap()
                .wait()
            {
                Ok(status) => println!("Command exited with code: {}", status.code().unwrap()),
                _ => eprintln!("Process failed to complete."),
            }
            None
        };

        if let Some(command) = command {
            self.processes.lock().unwrap().push(command);
        }
    }

    pub fn len(&self) -> usize {
        self.processes.lock().unwrap().len()
    }
}
