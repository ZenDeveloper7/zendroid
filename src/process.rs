use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex, mpsc::Sender};
use std::thread;

#[derive(Debug)]
pub enum ProcessEvent {
    Started { command: String },
    Output(String),
    Finished { success: bool, summary: String },
}

#[derive(Debug, Default)]
pub struct LogState {
    pub lines: Vec<String>,
    pub scroll: usize,
}

impl LogState {
    pub fn push(&mut self, line: impl Into<String>) {
        self.lines.push(line.into());
        if self.lines.len() > 4000 {
            let overflow = self.lines.len() - 4000;
            self.lines.drain(0..overflow);
        }
    }

    pub fn clear(&mut self) {
        self.lines.clear();
        self.scroll = 0;
    }
}

#[derive(Debug, Default)]
pub struct ProcessHandle {
    pub child: Option<Arc<Mutex<Child>>>,
    pub command_display: Option<String>,
}

impl ProcessHandle {
    pub fn is_running(&self) -> bool {
        self.child.is_some()
    }

    pub fn clear(&mut self) {
        self.child = None;
        self.command_display = None;
    }

    pub fn cancel(&mut self) -> Result<(), String> {
        let Some(child) = self.child.as_ref() else {
            return Err("no running process".to_string());
        };
        child
            .lock()
            .map_err(|_| "failed to lock running process".to_string())?
            .kill()
            .map_err(|err| format!("failed to cancel process: {err}"))
    }
}

pub fn spawn_command(
    program: &Path,
    args: &[String],
    cwd: &Path,
    tx: Sender<ProcessEvent>,
) -> Result<Arc<Mutex<Child>>, String> {
    let mut command = Command::new(program);
    command
        .args(args)
        .current_dir(cwd)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let display = format!(
        "{} {}",
        program.display(),
        args.iter()
            .map(|arg| arg.as_str())
            .collect::<Vec<_>>()
            .join(" ")
    );

    let mut child = command
        .spawn()
        .map_err(|err| format!("failed to start {display}: {err}"))?;
    let stdout = child.stdout.take();
    let stderr = child.stderr.take();
    let shared = Arc::new(Mutex::new(child));

    let _ = tx.send(ProcessEvent::Started {
        command: display.clone(),
    });

    if let Some(stdout) = stdout {
        let tx = tx.clone();
        thread::spawn(move || {
            let reader = BufReader::new(stdout);
            for line in reader.lines().map_while(Result::ok) {
                let _ = tx.send(ProcessEvent::Output(line));
            }
        });
    }

    if let Some(stderr) = stderr {
        let tx = tx.clone();
        thread::spawn(move || {
            let reader = BufReader::new(stderr);
            for line in reader.lines().map_while(Result::ok) {
                let _ = tx.send(ProcessEvent::Output(line));
            }
        });
    }

    let waiter = shared.clone();
    thread::spawn(move || {
        let result = waiter.lock().ok().and_then(|mut child| child.wait().ok());
        let (success, summary) = match result {
            Some(status) => (
                status.success(),
                format!("process exited with status {status}"),
            ),
            None => (false, "process ended unexpectedly".to_string()),
        };
        let _ = tx.send(ProcessEvent::Finished { success, summary });
    });

    Ok(shared)
}
