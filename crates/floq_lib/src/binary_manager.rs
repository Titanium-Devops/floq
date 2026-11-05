// binary_manager — download / verify (SHA-256) / decompress / install agent
// binaries, and define the agent CLI contract (architecture §14.4).

use std::path::Path;

/// Supported coding-agent CLIs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentKind {
    Claude,
    Codex,
    Cursor,
    OpenCode,
}

impl AgentKind {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "claude" | "claude-code" => Some(Self::Claude),
            "codex" => Some(Self::Codex),
            "cursor" | "cursor-agent" => Some(Self::Cursor),
            "opencode" => Some(Self::OpenCode),
            _ => None,
        }
    }

    pub fn binary(&self) -> &'static str {
        match self {
            Self::Claude => "claude",
            Self::Codex => "codex",
            Self::Cursor => "cursor-agent",
            Self::OpenCode => "opencode",
        }
    }

    /// CLI args for a one-shot prompt run (approximate; verify per CLI version).
    pub fn prompt_args(&self, prompt: &str) -> Vec<String> {
        match self {
            Self::Claude => vec!["-p".into(), prompt.into()],
            Self::Codex => vec!["exec".into(), prompt.into()],
            Self::Cursor => vec![prompt.into()],
            Self::OpenCode => vec!["run".into(), prompt.into()],
        }
    }

    /// Shell command to install the agent CLI (approximate; verify per CLI).
    pub fn install_command(&self) -> &'static str {
        match self {
            Self::Claude => "npm install -g @anthropic-ai/claude-code",
            Self::Codex => "npm install -g @openai/codex",
            Self::Cursor => "curl -fsSL https://cursor.com/install | bash",
            Self::OpenCode => "npm install -g opencode-ai",
        }
    }
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::Digest;
    let mut h = sha2::Sha256::new();
    h.update(bytes);
    h.finalize().iter().map(|b| format!("{b:02x}")).collect()
}

/// Download `url` to `dest`, optionally verifying against `expected_sha256`.
pub async fn download_and_verify(
    url: &str,
    dest: &Path,
    expected_sha256: Option<&str>,
) -> Result<(), String> {
    let bytes = reqwest::get(url)
        .await
        .map_err(|e| format!("download failed: {e}"))?
        .error_for_status()
        .map_err(|e| e.to_string())?
        .bytes()
        .await
        .map_err(|e| e.to_string())?;

    if let Some(expected) = expected_sha256 {
        let actual = sha256_hex(&bytes);
        if !actual.eq_ignore_ascii_case(expected) {
            return Err(format!("sha256 mismatch: expected {expected}, got {actual}"));
        }
    }

    tokio::fs::write(dest, &bytes).await.map_err(|e| e.to_string())
}

/// Decompress a gzip file (`src`) to `dest`. (zstd is a later addition.)
pub fn decompress_gzip(src: &Path, dest: &Path) -> Result<(), String> {
    use std::io::Read;
    let file = std::fs::File::open(src).map_err(|e| e.to_string())?;
    let mut gz = flate2::read::GzDecoder::new(file);
    let mut out = Vec::new();
    gz.read_to_end(&mut out).map_err(|e| e.to_string())?;
    std::fs::write(dest, &out).map_err(|e| e.to_string())
}

#[cfg(unix)]
pub fn make_executable(path: &Path) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;
    let mut perms = std::fs::metadata(path).map_err(|e| e.to_string())?.permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(path, perms).map_err(|e| e.to_string())
}

#[cfg(not(unix))]
pub fn make_executable(_path: &Path) -> Result<(), String> {
    Ok(())
}
