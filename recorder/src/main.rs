mod audio;
mod cli;
mod session;
mod signals;
mod transcription;

use anyhow::{bail, Result};
use clap::Parser;
use cli::{Cli, Command};
use ringbuf::traits::Split;
use ringbuf::HeapRb;
use std::os::unix::io::AsRawFd;
use std::path::Path;

const RING_BUFFER_SAMPLES: usize = 96_000; // 2 seconds at 48kHz

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("domino_recorder=info".parse()?),
        )
        .with_writer(std::io::stderr)
        .init();

    let cli = Cli::parse();

    match cli.command {
        Command::Start { out_dir } => cmd_start(out_dir.as_deref()),
        Command::Stop => cmd_stop(),
        Command::Status => cmd_status(),
        Command::Doctor => cmd_doctor(),
    }
}

fn cmd_start(out_dir: Option<&Path>) -> Result<()> {
    let (session_dir, started_at) = session::prepare_session(out_dir)?;
    let opus_path = session_dir.join("meeting.opus");
    let log_path = session_dir.join("recorder.log");

    match unsafe { libc::fork() } {
        -1 => bail!("fork failed: {}", std::io::Error::last_os_error()),
        0 => {
            // === Child (daemon) ===
            unsafe {
                libc::setsid();
            }

            // Redirect stdout/stderr to log file, close stdin
            let log_file = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&log_path)?;
            let log_fd = log_file.as_raw_fd();
            unsafe {
                libc::dup2(log_fd, libc::STDOUT_FILENO);
                libc::dup2(log_fd, libc::STDERR_FILENO);
                libc::close(libc::STDIN_FILENO);
            }
            drop(log_file);

            let shutdown = signals::shutdown_flag()?;

            // Mic capture — may block on the macOS TCC prompt on first run.
            let mic_rb = HeapRb::<f32>::new(RING_BUFFER_SAMPLES);
            let (mic_prod, mic_cons) = mic_rb.split();
            let mic = audio::mic::start_mic_capture(mic_prod)?;

            // System capture — required. A meeting recording without the
            // other side is a silent recording, so we refuse to start.
            // The parent detects the daemon exit and surfaces the first-run
            // setup instructions via wait_for_daemon_ready.
            let sys_rb = HeapRb::<f32>::new(RING_BUFFER_SAMPLES);
            let (sys_prod, sys_cons) = sys_rb.split();
            let system_capture = start_system(sys_prod).map_err(|e| {
                tracing::error!(error = %e, "system audio capture failed to start");
                anyhow::anyhow!(
                    "Screen & System Audio Recording permission is required but could not be acquired: {e}"
                )
            })?;
            let system_dropped = system_capture.dropped_samples.clone();
            let system = Some(system_capture);
            let system_cons = Some(sys_cons);

            // Audio captures are up. Writing the PID file is the parent's
            // readiness signal — do it here, AFTER captures started, so the
            // parent never reports a false success when mic setup fails.
            session::write_pid_file(std::process::id(), &session_dir, &started_at)?;

            tracing::info!(
                session_dir = %session_dir.display(),
                pid = std::process::id(),
                "daemon ready"
            );

            let encoder_handle = audio::encoder::spawn_encoder(
                mic_cons,
                system_cons,
                opus_path,
                shutdown.clone(),
                mic.dropped_samples.clone(),
                system_dropped,
            )?;

            // Wait for shutdown signal
            while !signals::is_shutdown(&shutdown) {
                std::thread::sleep(std::time::Duration::from_millis(100));
            }

            tracing::info!("shutdown signal received, stopping capture");
            drop(mic.stream);
            if let Some(sys) = system {
                sys.stop();
            }

            // Wait for encoder to flush and finalize the file
            match encoder_handle.join() {
                Ok(Ok(())) => tracing::info!("encoder finished cleanly"),
                Ok(Err(e)) => tracing::error!("encoder error: {e:#}"),
                Err(_) => tracing::error!("encoder thread panicked"),
            }

            session::remove_pid_file()?;
            tracing::info!("daemon exiting");

            std::process::exit(0);
        }
        child_pid => {
            // === Parent ===
            wait_for_daemon_ready(child_pid, &log_path)?;

            let info = session::read_active_session()?
                .ok_or_else(|| anyhow::anyhow!("daemon reported ready but PID file is missing"))?;
            let json = serde_json::to_string(&info)?;
            println!("{json}");
            std::process::exit(0);
        }
    }
}

/// Block until the daemon either (a) writes the PID file (ready), or
/// (b) exits before doing so (failed — usually TCC permission denied).
/// Returns Err on failure with a first-run-setup message and the tail of
/// the daemon's log for debugging.
fn wait_for_daemon_ready(child_pid: libc::pid_t, log_path: &Path) -> Result<()> {
    let pid_path = session::pid_file_path()?;
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);

    loop {
        let mut status: libc::c_int = 0;
        let ret = unsafe { libc::waitpid(child_pid, &mut status, libc::WNOHANG) };
        if ret == child_pid {
            let tail = read_log_tail(log_path, 20);
            bail!(
                "recorder failed to start.\n\n\
                 First-run setup on macOS (both permissions are required):\n  \
                 1. System Settings → Privacy & Security → Microphone: enable Claude Code (or your terminal).\n  \
                 2. System Settings → Privacy & Security → Screen & System Audio Recording: enable the same app.\n  \
                 3. Quit and relaunch Claude Code — macOS caches permissions per process, so the running app can't see grants made after it launched.\n  \
                 4. Run /mstart again.\n\n\
                 If macOS never prompted for Screen & System Audio Recording, its privacy database has a stale decision.\n\
                 Reset it, then retry from step 1:\n  \
                 tccutil reset ScreenCapture\n  \
                 tccutil reset Microphone\n\n\
                 Daemon log tail ({}):\n{}",
                log_path.display(),
                tail
            );
        }

        if pid_path.exists() {
            return Ok(());
        }

        if std::time::Instant::now() > deadline {
            unsafe {
                libc::kill(child_pid, libc::SIGTERM);
            }
            bail!(
                "recorder daemon did not report ready within 30 seconds. \
                 If a macOS permission dialog is waiting for input, dismiss it and re-run /mstart. \
                 Otherwise see {}",
                log_path.display()
            );
        }

        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}

fn read_log_tail(path: &Path, lines: usize) -> String {
    match std::fs::read_to_string(path) {
        Ok(content) => {
            let all: Vec<&str> = content.lines().collect();
            let start = all.len().saturating_sub(lines);
            all[start..].join("\n")
        }
        Err(_) => format!("  (log at {} not readable)", path.display()),
    }
}

fn cmd_stop() -> Result<()> {
    let info = session::stop_session()?;

    let opus_path = info.session_dir.join("meeting.opus");
    if !opus_path.exists() {
        println!(
            "Session stopped: {} (no audio file produced)",
            info.session_dir.display()
        );
        return Ok(());
    }

    let size_mb = std::fs::metadata(&opus_path)?.len() as f64 / (1024.0 * 1024.0);

    match transcription::run_on_session(&info.session_dir) {
        Ok(outcome) => {
            println!("Saved:");
            println!("  {} ({:.1} MB)", opus_path.display(), size_mb);
            println!(
                "  {} ({} segments, {:.0}s audio, {:.0}s wall, {})",
                outcome.transcript_path.display(),
                outcome.segment_count,
                outcome.duration_sec,
                outcome.wall_sec,
                outcome.accelerator,
            );
            Ok(())
        }
        Err(error) => {
            eprintln!("Transcription failed: {error:#}");
            eprintln!("Audio is preserved at: {}", opus_path.display());
            eprintln!(
                "Logs: {}",
                info.session_dir.join("transcription.log").display()
            );
            std::process::exit(2);
        }
    }
}

fn cmd_status() -> Result<()> {
    match session::read_active_session()? {
        Some(info) => {
            let json = serde_json::to_string(&info)?;
            println!("{json}");
        }
        None => {
            println!("{{}}");
        }
    }
    Ok(())
}

fn cmd_doctor() -> Result<()> {
    println!("Domino Recorder — Health Check");
    println!("  (doctor checks will be implemented in Phase 4)");
    Ok(())
}

#[cfg(target_os = "macos")]
fn start_system(producer: ringbuf::HeapProd<f32>) -> Result<audio::system::SystemCapture> {
    audio::system::start_system_capture(producer)
}

#[cfg(not(target_os = "macos"))]
fn start_system(_producer: ringbuf::HeapProd<f32>) -> Result<NoSystemCapture> {
    bail!("system audio capture is only supported on macOS")
}

#[cfg(not(target_os = "macos"))]
struct NoSystemCapture {
    pub dropped_samples: std::sync::Arc<std::sync::atomic::AtomicU64>,
}

#[cfg(not(target_os = "macos"))]
impl NoSystemCapture {
    fn stop(self) {}
}
