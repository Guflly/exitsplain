use std::io::{self, Read, Write};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;

#[derive(Debug)]
pub struct RunOutcome {
    pub exit_code: i32,
    pub captured: String,
    pub truncated: bool,
}

#[derive(Clone, Copy)]
enum Stream {
    Stdout,
    Stderr,
}

pub fn run(command: &[String], max_log_bytes: usize) -> io::Result<RunOutcome> {
    let (program, arguments) = command
        .split_first()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "no command provided"))?;

    let mut child = Command::new(program)
        .args(arguments)
        .stdin(Stdio::inherit())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| io::Error::other("failed to capture stdout"))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| io::Error::other("failed to capture stderr"))?;

    let (sender, receiver) = mpsc::channel();
    pump(stdout, Stream::Stdout, sender.clone());
    pump(stderr, Stream::Stderr, sender.clone());
    drop(sender);

    let mut capture = BoundedCapture::new(max_log_bytes);
    let mut terminal_stdout = io::stdout().lock();
    let mut terminal_stderr = io::stderr().lock();

    for (stream, bytes) in receiver {
        match stream {
            Stream::Stdout => {
                terminal_stdout.write_all(&bytes)?;
                terminal_stdout.flush()?;
            }
            Stream::Stderr => {
                terminal_stderr.write_all(&bytes)?;
                terminal_stderr.flush()?;
            }
        }
        capture.push(&bytes);
    }

    let status = child.wait()?;
    Ok(RunOutcome {
        exit_code: status.code().unwrap_or(1),
        captured: capture.finish(),
        truncated: capture.truncated,
    })
}

fn pump(
    reader: impl Read + Send + 'static,
    stream: Stream,
    sender: mpsc::Sender<(Stream, Vec<u8>)>,
) {
    thread::spawn(move || {
        let mut reader = reader;
        let mut buffer = [0_u8; 8192];
        loop {
            match reader.read(&mut buffer) {
                Ok(0) => break,
                Ok(length) => {
                    if sender.send((stream, buffer[..length].to_vec())).is_err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    });
}

struct BoundedCapture {
    limit: usize,
    head: Vec<u8>,
    tail: Vec<u8>,
    truncated: bool,
}

impl BoundedCapture {
    fn new(limit: usize) -> Self {
        Self {
            limit,
            head: Vec::new(),
            tail: Vec::new(),
            truncated: false,
        }
    }

    fn push(&mut self, bytes: &[u8]) {
        if !self.truncated {
            self.head.extend_from_slice(bytes);
            if self.head.len() <= self.limit {
                return;
            }

            self.truncated = true;
            let head_limit = self.limit / 2;
            let overflow = self.head.split_off(head_limit);
            self.tail = last_bytes(&overflow, self.limit - head_limit);
            return;
        }

        self.tail.extend_from_slice(bytes);
        let tail_limit = self.limit - (self.limit / 2);
        if self.tail.len() > tail_limit {
            let remove = self.tail.len() - tail_limit;
            self.tail.drain(..remove);
        }
    }

    fn finish(&self) -> String {
        if !self.truncated {
            return String::from_utf8_lossy(&self.head).into_owned();
        }

        format!(
            "{}\n[... middle of log omitted ...]\n{}",
            String::from_utf8_lossy(&self.head),
            String::from_utf8_lossy(&self.tail)
        )
    }
}

fn last_bytes(bytes: &[u8], limit: usize) -> Vec<u8> {
    bytes[bytes.len().saturating_sub(limit)..].to_vec()
}
