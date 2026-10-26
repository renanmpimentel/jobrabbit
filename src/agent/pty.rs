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
use std::sync::mpsc as std_mpsc;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver};

/// Handle for a process running in a PTY. `kill` terminates it early; `wait`
/// blocks until it has exited.
pub struct PtyHandle {
    killer: Box<dyn portable_pty::ChildKiller + Send + Sync>,
    exit_rx: std_mpsc::Receiver<u32>,
}

impl PtyHandle {
    /// Kill the process.
    pub fn kill(&mut self) -> Result<()> {
        self.killer.kill().context("kill process in PTY")?;
        Ok(())
    }

    /// Wait for the process to terminate and return the exit code.
    pub fn wait(&mut self) -> Result<u32> {
        self.exit_rx.recv().context("wait for process in PTY")
    }
}

/// Strips ANSI CSI (`ESC [ ... final-byte`) and OSC (`ESC ] ... BEL`/`ESC \`)
/// sequences from a line.
///
/// On Windows, ConPTY and console programs (e.g. `sh.exe` setting the window
/// title) prepend/inject these sequences into stdout — bytes that have no
/// Unix PTY equivalent and would otherwise corrupt a stream-json line enough
/// to fail JSON parsing.
fn strip_ansi_csi(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
        if c != '\u{1b}' {
            out.push(c);
            continue;
        }
        match chars.peek() {
            Some(&'[') => {
                chars.next(); // consume '['
                for next in chars.by_ref() {
                    if ('\u{40}'..='\u{7e}').contains(&next) {
                        break; // final byte of the CSI sequence
                    }
                }
            }
            Some(&']') => {
                chars.next(); // consume ']'
                while let Some(next) = chars.next() {
                    if next == '\u{07}' {
                        break; // BEL terminator
                    }
                    if next == '\u{1b}' && chars.peek() == Some(&'\\') {
                        chars.next(); // consume the trailing '\' of ST (ESC \)
                        break;
                    }
                }
            }
            _ => {} // bare/unrecognized ESC: drop it
        }
    }
    out
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
            // On Windows, ConPTY is a real terminal emulator: it reflows output at
            // the PTY's column width, inserting hard line breaks into lines wider
            // than `cols`. `claude`'s stream-json lines routinely run into the
            // thousands of characters (e.g. the system/init line), so a narrow
            // width silently shreds a single JSON line into multiple unparseable
            // fragments. A near-max column count keeps that from happening in
            // practice. Unix PTYs don't reflow, so this is a no-op there.
            cols: 8192,
            pixel_width: 0,
            pixel_height: 0,
        })
        .context("open PTY")?;

    let mut cmd = CommandBuilder::new(program);
    cmd.args(args);
    if let Some(c) = cwd {
        cmd.cwd(c);
    }

    let mut child = pair
        .slave
        .spawn_command(cmd)
        .with_context(|| format!("spawn `{program}` in PTY"))?;
    // The slave is no longer needed on our side.
    drop(pair.slave);

    let killer = child.clone_killer();
    let reader = pair.master.try_clone_reader().context("clone PTY reader")?;
    let (tx, rx) = unbounded_channel::<String>();

    // Blocking read thread → sends each line over the tokio channel.
    std::thread::spawn(move || {
        let buf = BufReader::new(reader);
        for line in buf.lines() {
            match line {
                Ok(l) => {
                    let l = strip_ansi_csi(l.trim_end_matches('\r'));
                    if tx.send(l).is_err() {
                        break; // receiver dropped
                    }
                }
                Err(_) => break, // EOF / process ended
            }
        }
    });

    let (exit_tx, exit_rx) = std_mpsc::channel();
    // On Windows, ConPTY keeps the pseudo-console's output pipe open until the
    // *master* is explicitly closed — a child exiting on its own does not make
    // the reader thread above see EOF. Without this thread to drop `pair.master`
    // right after the child exits, the read loop blocks forever even though the
    // process is long gone. See wezterm/wezterm#463 and #1396.
    std::thread::spawn(move || {
        let code = child.wait().map(|s| s.exit_code()).unwrap_or(u32::MAX);
        drop(pair.master);
        let _ = exit_tx.send(code);
    });

    Ok((PtyHandle { killer, exit_rx }, rx))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_ansi_csi_removes_terminal_mode_sequences() {
        assert_eq!(
            strip_ansi_csi("\u{1b}[?9001h\u{1b}[?1004h\u{1b}[?25l\u{1b}[2J\u{1b}[m\u{1b}[Hline1"),
            "line1"
        );
        assert_eq!(strip_ansi_csi("plain text"), "plain text");
    }

    #[test]
    fn strip_ansi_csi_removes_osc_window_title() {
        assert_eq!(
            strip_ansi_csi("\u{1b}]0;C:\\Program Files\\Git\\bin\\sh.EXE\u{07}line2"),
            "line2"
        );
    }

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
