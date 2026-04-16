#![allow(dead_code)]
// Callers in cmd_stop wire up in Phase 5; until then the binary doesn't touch
// these items, only the test suite does.

//! Discover, download, and verify the whisper.cpp ggml-small.en model.
//!
//! The model lives at `~/.domino/models/ggml-small.en.bin` (~465 MB).
//! The plugin is expected to pre-download it at install time; this module
//! is the safety net when the file is missing or corrupted at transcribe time.
//!
//! ## SHA256 pinning
//!
//! `MODEL_SHA256_HEX` should be set to the known SHA256 of the official
//! ggml-small.en.bin release before shipping. Until then, it is the empty
//! string, which disables verification and logs a warning on every check.
//! To populate it:
//!   1. Run the binary once to download the model to `~/.domino/models/`.
//!   2. `shasum -a 256 ~/.domino/models/ggml-small.en.bin`
//!   3. Paste the hex digest into `MODEL_SHA256_HEX` and re-build.

use anyhow::{bail, Context, Result};
use indicatif::{ProgressBar, ProgressDrawTarget, ProgressStyle};
use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};

pub const MODEL_FILENAME: &str = "ggml-small.en.bin";
pub const MODEL_URL: &str =
    "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.en.bin";

/// SHA256 of ggml-small.en.bin as published by ggerganov/whisper.cpp.
/// Empty string => verification is skipped (with a warning). See module docs
/// for the ritual to fill this in before shipping.
pub const MODEL_SHA256_HEX: &str = "";

/// Resolve `~/.domino/models/`, creating it with mode 0o700 on unix if missing.
pub fn models_dir() -> Result<PathBuf> {
    let home = dirs::home_dir().context("could not determine home directory")?;
    let dir = home.join(".domino").join("models");
    if !dir.exists() {
        fs::create_dir_all(&dir).with_context(|| format!("failed to create {}", dir.display()))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&dir, fs::Permissions::from_mode(0o700))?;
        }
    }
    Ok(dir)
}

/// Ensure the model is present at `~/.domino/models/ggml-small.en.bin` and
/// SHA256-verified; download it if not. Returns the path to the verified file.
pub fn ensure_model_available() -> Result<PathBuf> {
    ensure_model_at(&models_dir()?, MODEL_URL, MODEL_SHA256_HEX)
}

/// Same as `ensure_model_available` but with the models directory, URL, and
/// expected SHA injected — exists for unit testing without touching real HOME
/// or hitting the network.
pub(crate) fn ensure_model_at(dir: &Path, url: &str, expected_sha: &str) -> Result<PathBuf> {
    if !dir.exists() {
        fs::create_dir_all(dir)?;
    }
    let path = dir.join(MODEL_FILENAME);

    if path.exists() {
        match verify_sha256(&path, expected_sha) {
            Ok(true) => {
                tracing::debug!(path = %path.display(), "model present and verified");
                return Ok(path);
            }
            Ok(false) => {
                tracing::warn!(
                    path = %path.display(),
                    "model SHA mismatch — removing and re-downloading"
                );
                fs::remove_file(&path).ok();
            }
            Err(e) => {
                tracing::warn!(error = %e, "model verification failed — removing and re-downloading");
                fs::remove_file(&path).ok();
            }
        }
    }

    download_with_progress(url, &path).context("failed to download model")?;

    if !verify_sha256(&path, expected_sha)? {
        fs::remove_file(&path).ok();
        bail!(
            "downloaded model at {} failed SHA256 verification",
            path.display()
        );
    }

    Ok(path)
}

/// Compute the SHA256 of `path` and compare to `expected_hex` (case-insensitive).
/// If `expected_hex` is empty, returns `true` with a warning — used during
/// bootstrap when we don't yet have a pinned hash.
pub(crate) fn verify_sha256(path: &Path, expected_hex: &str) -> Result<bool> {
    if expected_hex.is_empty() {
        tracing::warn!(
            path = %path.display(),
            "MODEL_SHA256_HEX is empty; skipping integrity verification"
        );
        return Ok(true);
    }

    let mut f = BufReader::new(File::open(path)?);
    let mut hasher = Sha256::new();
    let mut buf = vec![0u8; 1 << 20];
    loop {
        let n = f.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    let got = hex::encode(hasher.finalize());
    Ok(got.eq_ignore_ascii_case(expected_hex))
}

/// Download `url` into `dest` with a progress bar. Resumes from `dest.part`
/// if an interrupted partial download is present.
fn download_with_progress(url: &str, dest: &Path) -> Result<()> {
    let part_path = partial_path_for(dest);
    let existing = fs::metadata(&part_path).map(|m| m.len()).unwrap_or(0);

    let mut req = ureq::get(url);
    if existing > 0 {
        req = req.set("Range", &format!("bytes={}-", existing));
    }
    let resp = req.call().context("model download request failed")?;

    let status = resp.status();
    // If we asked for a range but the server replied 200 OK, it ignored the
    // Range header and is sending the whole file — truncate and restart.
    let restart = existing > 0 && status == 200;
    let effective_existing = if restart { 0 } else { existing };

    let remaining: u64 = resp
        .header("Content-Length")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    let total = remaining + effective_existing;

    let pb = ProgressBar::with_draw_target(Some(total), ProgressDrawTarget::stdout());
    pb.set_style(
        ProgressStyle::with_template(
            "downloading ggml-small.en.bin [{bar:40}] {bytes}/{total_bytes} ({eta})",
        )?
        .progress_chars("#>-"),
    );
    pb.set_position(effective_existing);

    let mut reader = resp.into_reader();
    let append = effective_existing > 0;
    let mut out = BufWriter::new(
        fs::OpenOptions::new()
            .create(true)
            .write(true)
            .append(append)
            .truncate(!append)
            .open(&part_path)?,
    );
    let mut buf = vec![0u8; 1 << 20];
    loop {
        let n = reader.read(&mut buf)?;
        if n == 0 {
            break;
        }
        out.write_all(&buf[..n])?;
        pb.inc(n as u64);
    }
    out.flush()?;
    drop(out);

    if let Some(parent) = dest.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)?;
        }
    }
    fs::rename(&part_path, dest)
        .with_context(|| format!("failed to finalize download at {}", dest.display()))?;
    pb.finish_with_message("model ready");
    Ok(())
}

fn partial_path_for(dest: &Path) -> PathBuf {
    let mut s = dest.as_os_str().to_os_string();
    s.push(".part");
    PathBuf::from(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tempdir(tag: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("domino-test-model-{}-{}", tag, std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn write_bytes(path: &Path, bytes: &[u8]) {
        let mut f = File::create(path).unwrap();
        f.write_all(bytes).unwrap();
    }

    fn sha256_of(bytes: &[u8]) -> String {
        let mut h = Sha256::new();
        h.update(bytes);
        hex::encode(h.finalize())
    }

    #[test]
    fn test_verify_sha256_match() {
        let dir = tempdir("verify-match");
        let path = dir.join("f.bin");
        let bytes = b"hello world";
        write_bytes(&path, bytes);
        let expected = sha256_of(bytes);
        assert!(verify_sha256(&path, &expected).unwrap());
        assert!(verify_sha256(&path, &expected.to_uppercase()).unwrap());
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_verify_sha256_mismatch() {
        let dir = tempdir("verify-mismatch");
        let path = dir.join("f.bin");
        write_bytes(&path, b"hello world");
        let wrong = "0".repeat(64);
        assert!(!verify_sha256(&path, &wrong).unwrap());
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_verify_sha256_empty_expected_skips_check() {
        let dir = tempdir("verify-empty");
        let path = dir.join("f.bin");
        write_bytes(&path, b"anything at all");
        assert!(verify_sha256(&path, "").unwrap());
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_verify_sha256_nonexistent_file_errors() {
        let dir = tempdir("verify-missing");
        let path = dir.join("no-such-file.bin");
        let res = verify_sha256(&path, &"0".repeat(64));
        assert!(res.is_err());
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_ensure_model_at_noop_when_present_and_verified() {
        let dir = tempdir("ensure-noop");
        let model_path = dir.join(MODEL_FILENAME);
        let bytes = b"dummy small.en contents for test";
        write_bytes(&model_path, bytes);
        let expected = sha256_of(bytes);

        // URL is intentionally unreachable; if the function tried to
        // download, the test would fail with a network/DNS error.
        let url = "https://invalid.example.invalid/should-not-be-fetched";

        let got = ensure_model_at(&dir, url, &expected).unwrap();
        assert_eq!(got, model_path);
        // File is untouched.
        let on_disk = fs::read(&model_path).unwrap();
        assert_eq!(on_disk, bytes);

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_ensure_model_at_empty_sha_is_noop_when_present() {
        // With verification disabled (empty expected SHA), any existing file
        // is accepted as-is and no download is attempted.
        let dir = tempdir("ensure-empty-sha");
        let model_path = dir.join(MODEL_FILENAME);
        write_bytes(&model_path, b"whatever");

        let url = "https://invalid.example.invalid/should-not-be-fetched";
        let got = ensure_model_at(&dir, url, "").unwrap();
        assert_eq!(got, model_path);

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_ensure_model_at_removes_corrupted_before_redownload() {
        // Sanity check that a SHA mismatch removes the file. We can't fully
        // exercise the download branch without network, so we stop just after
        // removal by pointing at an unreachable URL and asserting the file
        // was deleted and the call errored.
        let dir = tempdir("ensure-corrupt");
        let model_path = dir.join(MODEL_FILENAME);
        write_bytes(&model_path, b"corrupted bytes");
        let wrong_sha = "0".repeat(64);

        let url = "https://invalid.example.invalid/should-not-be-fetched";
        let res = ensure_model_at(&dir, url, &wrong_sha);
        assert!(res.is_err(), "download to invalid host should fail");
        assert!(
            !model_path.exists(),
            "corrupted file should have been removed before the download attempt"
        );

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_partial_path_for() {
        let p = PathBuf::from("/tmp/ggml-small.en.bin");
        assert_eq!(
            partial_path_for(&p),
            PathBuf::from("/tmp/ggml-small.en.bin.part")
        );
    }

    #[test]
    fn test_models_dir_is_under_domino() {
        let dir = models_dir().unwrap();
        assert!(dir.ends_with(".domino/models"));
    }
}
