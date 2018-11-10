use std::process::{id, Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::io;

pub struct ProcessPool {
    processes: Arc<Mutex<Vec<Child>>>,
    thread_handle: Option<std::thread::JoinHandle<()>>,
    tx: std::sync::mpsc::Sender<()>,
    last_exit_code: Arc<Mutex<Option<i32>>>,
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
        let code = Arc::new(Mutex::new(None));
        let (tx, rx) = std::sync::mpsc::channel();
        let cloned = processes.clone();
        let code_cloned = code.clone();

        ProcessPool {
            processes: processes,
            thread_handle: Some(std::thread::spawn(move || loop {
                let mut processes = cloned.lock().unwrap();

                processes.drain_filter(|process| {
                    let finished = process.try_wait();

                    match finished {
                        Ok(Some(status)) => {
                            println!("Command exited with code: {}", status.code().unwrap());
                            *code_cloned.lock().unwrap() = status.code();
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
            last_exit_code: code,
        }
    }

    pub fn add(&mut self, command: &str, mut args: Vec<&str>) -> io::Result<()> {

        let backgrounded = match args.last() {
            Some(&arg) if arg == "&" => {
                args.pop();
                true
            },
            _ => false
        };

        let command = if backgrounded {
            Some(
                Command::new(command)
                    .args(args)
                    .stdin(Stdio::null())
                    .spawn()?,
            )
        } else {
            let mut command = Command::new(command)
                .args(args)
                .stdin(Stdio::null())
                .spawn()?;

            match command.wait()
            {
                Ok(status) => println!("Command exited with code: {}", status.code().unwrap()),
                _ => eprintln!("Process failed to complete."),
            }
            None
        };

        if let Some(command) = command {
            self.processes.lock().unwrap().push(command);
        }

        Ok(())
    }

    pub fn len(&self) -> usize {
        self.processes.lock().unwrap().len()
    }

    pub fn last_exit_code(&self) -> Option<i32> {
        *self.last_exit_code.lock().unwrap()
    }
}
