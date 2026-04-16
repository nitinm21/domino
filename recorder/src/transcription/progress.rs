use anyhow::Result;
use indicatif::{ProgressBar, ProgressDrawTarget, ProgressStyle};
use std::fs::File;
use std::io::{self, Write};
use std::path::Path;
use std::sync::{Arc, Mutex};
use tracing::Level;

pub fn overall_bar(duration_sec: f64) -> ProgressBar {
    let total_ms = ((duration_sec * 1000.0).round() as u64).saturating_mul(2);
    let pb = ProgressBar::with_draw_target(Some(total_ms), ProgressDrawTarget::stdout());
    pb.set_style(
        ProgressStyle::with_template("{msg:<40} [{bar:30}] {percent}% (ETA {eta})")
            .expect("valid progress template")
            .progress_chars("#>-"),
    );
    pb
}

pub struct LogGuard {
    _inner: tracing::subscriber::DefaultGuard,
}

/// Route transcription tracing into `transcription.log` and stderr for the
/// duration of the run without mutating the process-wide subscriber.
pub fn init_log_file(path: &Path) -> Result<LogGuard> {
    let file = Arc::new(Mutex::new(File::create(path)?));
    let writer_file = Arc::clone(&file);

    let subscriber = tracing_subscriber::fmt()
        .with_ansi(false)
        .with_max_level(Level::INFO)
        .with_writer(move || TeeWriter {
            file: Arc::clone(&writer_file),
            stderr: io::stderr(),
        })
        .finish();

    let guard = tracing::subscriber::set_default(subscriber);
    Ok(LogGuard { _inner: guard })
}

struct TeeWriter {
    file: Arc<Mutex<File>>,
    stderr: io::Stderr,
}

impl Write for TeeWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.stderr.write_all(buf)?;
        let mut file = self
            .file
            .lock()
            .map_err(|_| io::Error::other("transcription log mutex poisoned"))?;
        file.write_all(buf)?;
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        self.stderr.flush()?;
        let mut file = self
            .file
            .lock()
            .map_err(|_| io::Error::other("transcription log mutex poisoned"))?;
        file.flush()
    }
}
