//! PTY wrapper (portable-pty) for spawning processes (the `claude`) and reading
//! stdout line by line asynchronously.
//!
//! The `claude` is spawned in a pseudo-terminal because Chrome integration and
//! some CLI prompts expect a TTY. Lines read are sent over a non-blocking `tokio`
//! channel (fed by a blocking IO thread).

use anyhow::{Context, Result};
use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use std::io::{BufRead, BufReader};
use std::path::Path;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver};

/// Handle for a process running in a PTY. Keep alive while the session lasts;
/// `drop`/`kill` terminate the process.
pub struct PtyHandle {
    child: Box<dyn portable_pty::Child + Send + Sync>,
    _master: Box<dyn portable_pty::MasterPty + Send>,
}

impl PtyHandle {
    /// Kill the process.
    pub fn kill(&mut self) -> Result<()> {
        self.child.kill().context("kill process in PTY")?;
        Ok(())
    }

    /// Wait for the process to terminate and return the exit code.
    pub fn wait(&mut self) -> Result<u32> {
        let status = self.child.wait().context("wait for process in PTY")?;
        Ok(status.exit_code())
    }
}

/// Spawn `program` with `args` in a PTY. Returns the handle and a receiver for stdout lines
/// (with `\n`/`\r` already removed).
pub fn spawn(
    program: &str,
    args: &[String],
    cwd: Option<&Path>,
) -> Result<(PtyHandle, UnboundedReceiver<String>)> {
    let pty_system = native_pty_system();
    let pair = pty_system
        .openpty(PtySize {
            rows: 40,
            cols: 140,
            pixel_width: 0,
            pixel_height: 0,
        })
        .context("open PTY")?;

    let mut cmd = CommandBuilder::new(program);
    cmd.args(args);
    if let Some(c) = cwd {
        cmd.cwd(c);
    }

    let child = pair
        .slave
        .spawn_command(cmd)
        .with_context(|| format!("spawn `{program}` in PTY"))?;
    // The slave is no longer needed on our side.
    drop(pair.slave);

    let reader = pair.master.try_clone_reader().context("clone PTY reader")?;
    let (tx, rx) = unbounded_channel::<String>();

    // Blocking read thread → sends each line over the tokio channel.
    std::thread::spawn(move || {
        let buf = BufReader::new(reader);
        for line in buf.lines() {
            match line {
                Ok(l) => {
                    let l = l.trim_end_matches('\r').to_string();
                    if tx.send(l).is_err() {
                        break; // receiver dropped
                    }
                }
                Err(_) => break, // EOF / process ended
            }
        }
    });

    Ok((
        PtyHandle {
            child,
            _master: pair.master,
        },
        rx,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn spawn_reads_lines_from_stdout() {
        // Does not depend on `claude`: validates the PTY mechanism + line reading.
        let (mut handle, mut rx) = spawn(
            "sh",
            &["-c".to_string(), "printf 'line1\\nline2\\n'".to_string()],
            None,
        )
        .expect("spawn should work");

        let l1 = rx.recv().await;
        let l2 = rx.recv().await;
        assert_eq!(l1.as_deref(), Some("line1"));
        assert_eq!(l2.as_deref(), Some("line2"));

        let _ = handle.wait();
    }
}
