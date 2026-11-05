// PTY spawn — minimal portable-pty wrapper. Terminal-history persistence is a
// later refinement.

use portable_pty::{native_pty_system, CommandBuilder, MasterPty, PtySize};
use std::io::{Read, Write};
use std::path::Path;

pub struct PtySession {
    master: Box<dyn MasterPty + Send>,
    writer: Box<dyn Write + Send>,
    child: Box<dyn portable_pty::Child + Send + Sync>,
}

impl PtySession {
    /// Run `sh -c <command>` in a PTY.
    pub fn spawn(command: &str, cwd: Option<&Path>) -> Result<Self, String> {
        Self::spawn_program("sh", &["-c", command], cwd)
    }

    /// Run the user's login shell (`$SHELL`, fallback `sh`) interactively.
    pub fn spawn_shell(cwd: Option<&Path>) -> Result<Self, String> {
        let shell = std::env::var("SHELL").unwrap_or_else(|_| "sh".to_string());
        Self::spawn_program(&shell, &[], cwd)
    }

    /// Spawn `program args…` in a PTY.
    pub fn spawn_program(program: &str, args: &[&str], cwd: Option<&Path>) -> Result<Self, String> {
        let system = native_pty_system();
        let pair = system
            .openpty(PtySize {
                rows: 24,
                cols: 80,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| e.to_string())?;

        let mut cmd = CommandBuilder::new(program);
        for a in args {
            cmd.arg(a);
        }
        if let Some(dir) = cwd {
            cmd.cwd(dir);
        }
        // Clean environment so login-shell configs (nvm, starship) don't misbehave.
        cmd.env("TERM", "xterm-256color");
        cmd.env_remove("npm_config_prefix");
        let child = pair.slave.spawn_command(cmd).map_err(|e| e.to_string())?;
        drop(pair.slave);

        let writer = pair.master.take_writer().map_err(|e| e.to_string())?;
        Ok(Self {
            master: pair.master,
            writer,
            child,
        })
    }

    pub fn write(&mut self, data: &[u8]) -> Result<(), String> {
        self.writer.write_all(data).map_err(|e| e.to_string())
    }

    pub fn read(&mut self, buf: &mut [u8]) -> Result<usize, String> {
        let mut reader = self.master.try_clone_reader().map_err(|e| e.to_string())?;
        reader.read(buf).map_err(|e| e.to_string())
    }

    pub fn resize(&mut self, rows: u16, cols: u16) -> Result<(), String> {
        self.master
            .resize(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| e.to_string())
    }

    pub fn kill(&mut self) -> Result<(), String> {
        self.child.kill().map_err(|e| e.to_string())
    }

    pub fn try_wait(&mut self) -> Option<portable_pty::ExitStatus> {
        self.child.try_wait().ok().flatten()
    }
}
