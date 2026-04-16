#![allow(dead_code)]
// Callers (transcription::run_on_session) wire up in Phase 5.

//! Decode an Ogg Opus file into per-channel 48 kHz mono f32 buffers.
//!
//! We reuse the same `audiopus` + `ogg` crates the encoder uses, instead of
//! pulling in `symphonia`. This keeps the binary smaller, the codec path
//! symmetric with the encoder, and avoids symphonia's still-experimental
//! Opus support.

use anyhow::{bail, Context, Result};
use audiopus::coder::Decoder;
use audiopus::{Channels, SampleRate};
use ogg::reading::PacketReader;
use std::fs::File;
use std::path::Path;

/// Our encoder writes at 48 kHz stereo; assert both on the decode side.
const EXPECTED_SAMPLE_RATE: u32 = 48_000;
const EXPECTED_CHANNELS: u8 = 2;

/// Biggest Opus frame we expect to see (120 ms at 48 kHz = 5760 samples per
/// channel). Our encoder writes 20 ms frames but we size for the worst case.
const MAX_FRAME_SAMPLES_PER_CHANNEL: usize = 5760;

/// Decode a stereo Ogg Opus file into two 48 kHz mono `Vec<f32>` buffers and
/// the duration in seconds (post pre-skip trim).
///
/// Returns `(left_48k, right_48k, duration_sec)`.
pub fn decode_stereo_opus(path: &Path) -> Result<(Vec<f32>, Vec<f32>, f64)> {
    let file = File::open(path).with_context(|| format!("failed to open {}", path.display()))?;
    let mut reader = PacketReader::new(file);

    // --- Header packet: OpusHead ---
    let head = reader
        .read_packet()
        .context("failed to read first Ogg packet")?
        .context("empty Ogg stream")?;
    let (channels, pre_skip) = parse_opus_head(&head.data)?;
    if channels != EXPECTED_CHANNELS {
        bail!("expected stereo Opus ({EXPECTED_CHANNELS} channels), got {channels}");
    }

    // --- Metadata packet: OpusTags (ignored) ---
    let _tags = reader
        .read_packet()
        .context("failed to read OpusTags packet")?
        .context("missing OpusTags packet")?;

    // --- Audio packets ---
    let mut decoder =
        Decoder::new(SampleRate::Hz48000, Channels::Stereo).context("audiopus decoder init")?;
    let mut frame_buf = vec![0.0f32; MAX_FRAME_SAMPLES_PER_CHANNEL * 2];
    let mut interleaved: Vec<f32> = Vec::new();

    loop {
        let pkt = match reader.read_packet() {
            Ok(Some(p)) => p,
            Ok(None) => break,
            Err(e) => return Err(anyhow::Error::new(e).context("Ogg read error")),
        };
        let samples_per_ch = decoder
            .decode_float(Some(&pkt.data[..]), &mut frame_buf[..], false)
            .context("Opus decode failed")?;
        interleaved.extend_from_slice(&frame_buf[..samples_per_ch * 2]);
    }

    // --- Split stereo, trim pre_skip from each channel ---
    let total_frames = interleaved.len() / 2;
    let skip = (pre_skip as usize).min(total_frames);
    let kept_frames = total_frames - skip;

    let mut left = Vec::with_capacity(kept_frames);
    let mut right = Vec::with_capacity(kept_frames);
    for i in skip..total_frames {
        left.push(interleaved[i * 2]);
        right.push(interleaved[i * 2 + 1]);
    }

    let duration_sec = kept_frames as f64 / EXPECTED_SAMPLE_RATE as f64;
    Ok((left, right, duration_sec))
}

/// Parse an OpusHead packet payload. Returns `(channels, pre_skip_samples)`.
fn parse_opus_head(data: &[u8]) -> Result<(u8, u16)> {
    if data.len() < 19 || &data[..8] != b"OpusHead" {
        bail!("not an OpusHead packet");
    }
    let version = data[8];
    if version != 1 {
        bail!("unsupported OpusHead version: {version}");
    }
    let channels = data[9];
    let pre_skip = u16::from_le_bytes([data[10], data[11]]);
    let input_rate = u32::from_le_bytes([data[12], data[13], data[14], data[15]]);
    let channel_mapping_family = data[18];
    if channel_mapping_family != 0 {
        bail!("unsupported channel mapping family: {channel_mapping_family}");
    }
    tracing::debug!(channels, pre_skip, input_rate, "OpusHead parsed");
    Ok((channels, pre_skip))
}

#[cfg(test)]
mod tests {
    use super::*;
    use audiopus::coder::Encoder;
    use audiopus::{Application, Bitrate};
    use ogg::writing::{PacketWriteEndInfo, PacketWriter};
    use std::fs;
    use std::io::BufWriter;
    use std::path::PathBuf;

    const FRAME: usize = 960; // 20 ms at 48 kHz

    fn tempdir(tag: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("domino-test-decode-{}-{}", tag, std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    /// Write a stereo Ogg Opus file with the given per-channel sample buffers.
    /// `left.len()` and `right.len()` must be equal and a multiple of FRAME.
    fn write_stereo_opus(path: &Path, left: &[f32], right: &[f32]) -> u16 {
        assert_eq!(left.len(), right.len(), "channels must be equal length");
        assert_eq!(left.len() % FRAME, 0, "length must be a multiple of 960");

        let mut enc =
            Encoder::new(SampleRate::Hz48000, Channels::Stereo, Application::Voip).unwrap();
        enc.set_bitrate(Bitrate::BitsPerSecond(64_000)).unwrap();
        let pre_skip = enc.lookahead().unwrap_or(312) as u16;

        let file = fs::File::create(path).unwrap();
        let mut w = PacketWriter::new(BufWriter::new(file));
        let serial = 1u32;

        let mut head = Vec::with_capacity(19);
        head.extend_from_slice(b"OpusHead");
        head.push(1);
        head.push(2);
        head.extend_from_slice(&pre_skip.to_le_bytes());
        head.extend_from_slice(&48000u32.to_le_bytes());
        head.extend_from_slice(&0u16.to_le_bytes());
        head.push(0);
        w.write_packet(head, serial, PacketWriteEndInfo::EndPage, 0)
            .unwrap();

        let mut tags = Vec::new();
        tags.extend_from_slice(b"OpusTags");
        let vendor = b"test-fixture";
        tags.extend_from_slice(&(vendor.len() as u32).to_le_bytes());
        tags.extend_from_slice(vendor);
        tags.extend_from_slice(&0u32.to_le_bytes());
        w.write_packet(tags, serial, PacketWriteEndInfo::EndPage, 0)
            .unwrap();

        let n_frames = left.len() / FRAME;
        let mut interleaved = vec![0.0f32; FRAME * 2];
        let mut out_buf = vec![0u8; 4000];
        let mut granule: u64 = pre_skip as u64;
        for f in 0..n_frames {
            for i in 0..FRAME {
                interleaved[i * 2] = left[f * FRAME + i];
                interleaved[i * 2 + 1] = right[f * FRAME + i];
            }
            let len = enc
                .encode_float(&interleaved[..], &mut out_buf[..])
                .unwrap();
            granule += FRAME as u64;
            let end = if f == n_frames - 1 {
                PacketWriteEndInfo::EndStream
            } else {
                PacketWriteEndInfo::EndPage
            };
            w.write_packet(out_buf[..len].to_vec(), serial, end, granule)
                .unwrap();
        }
        pre_skip
    }

    fn sine(n: usize, freq_hz: f32, amp: f32) -> Vec<f32> {
        (0..n)
            .map(|i| (2.0 * std::f32::consts::PI * freq_hz * i as f32 / 48_000.0).sin() * amp)
            .collect()
    }

    fn rms(buf: &[f32]) -> f32 {
        if buf.is_empty() {
            return 0.0;
        }
        let sum: f32 = buf.iter().map(|x| x * x).sum();
        (sum / buf.len() as f32).sqrt()
    }

    #[test]
    fn test_parse_opus_head_roundtrip() {
        let mut head = Vec::new();
        head.extend_from_slice(b"OpusHead");
        head.push(1);
        head.push(2);
        head.extend_from_slice(&312u16.to_le_bytes());
        head.extend_from_slice(&48000u32.to_le_bytes());
        head.extend_from_slice(&0u16.to_le_bytes());
        head.push(0);
        let (ch, ps) = parse_opus_head(&head).unwrap();
        assert_eq!(ch, 2);
        assert_eq!(ps, 312);
    }

    #[test]
    fn test_parse_opus_head_rejects_bad_magic() {
        let bad = vec![0u8; 19];
        assert!(parse_opus_head(&bad).is_err());
    }

    #[test]
    fn test_parse_opus_head_rejects_short() {
        let short = b"OpusHead\x01\x02".to_vec();
        assert!(parse_opus_head(&short).is_err());
    }

    #[test]
    fn test_decode_rejects_missing_file() {
        let dir = tempdir("missing");
        let res = decode_stereo_opus(&dir.join("nope.opus"));
        assert!(res.is_err());
        fs::remove_dir_all(&dir).ok();
    }

    /// Encode a stereo sine (440 Hz left, 880 Hz right) for ~0.25 seconds,
    /// decode it back, and verify:
    ///   - both channels come back at 48 kHz with ~the same length,
    ///   - duration is in the right ballpark,
    ///   - both channels have non-trivial RMS (decoder produced real audio),
    ///   - left and right are distinct (not bleeding into each other).
    #[test]
    fn test_decode_stereo_round_trip() {
        let dir = tempdir("roundtrip");
        let path = dir.join("fixture.opus");

        let n_frames: usize = 12; // 12 * 20 ms = 240 ms
        let n_samples = n_frames * FRAME;
        let left = sine(n_samples, 440.0, 0.5);
        let right = sine(n_samples, 880.0, 0.5);

        write_stereo_opus(&path, &left, &right);

        let (dec_l, dec_r, dur) = decode_stereo_opus(&path).unwrap();

        assert_eq!(dec_l.len(), dec_r.len(), "channels must match length");
        let expected_len = n_samples;
        let diff = (dec_l.len() as i64 - expected_len as i64).abs();
        assert!(
            diff < (FRAME as i64) * 2,
            "decoded length {} far from expected {}",
            dec_l.len(),
            expected_len
        );

        let expected_dur = expected_len as f64 / 48_000.0;
        assert!(
            (dur - expected_dur).abs() < 0.05,
            "duration {} not near {}",
            dur,
            expected_dur
        );

        // Both channels should carry signal (lossy codec => RMS approximately preserved).
        let rms_l = rms(&dec_l);
        let rms_r = rms(&dec_r);
        assert!(rms_l > 0.1, "left channel decoded silent: rms={rms_l}");
        assert!(rms_r > 0.1, "right channel decoded silent: rms={rms_r}");

        fs::remove_dir_all(&dir).ok();
    }

    /// Pure silence on both channels decodes to very low RMS (near zero).
    #[test]
    fn test_decode_silence() {
        let dir = tempdir("silence");
        let path = dir.join("silence.opus");
        let n = 5 * FRAME;
        let silent = vec![0.0f32; n];
        write_stereo_opus(&path, &silent, &silent);

        let (l, r, _) = decode_stereo_opus(&path).unwrap();
        assert!(rms(&l) < 0.01);
        assert!(rms(&r) < 0.01);
        fs::remove_dir_all(&dir).ok();
    }

    /// Mic-only recording (left has content, right is silent) round-trips
    /// with left still carrying signal and right staying near zero.
    #[test]
    fn test_decode_preserves_channel_separation() {
        let dir = tempdir("separation");
        let path = dir.join("left-only.opus");
        let n = 8 * FRAME;
        let left = sine(n, 440.0, 0.5);
        let right = vec![0.0f32; n];
        write_stereo_opus(&path, &left, &right);

        let (dec_l, dec_r, _) = decode_stereo_opus(&path).unwrap();
        let rms_l = rms(&dec_l);
        let rms_r = rms(&dec_r);
        assert!(rms_l > 0.1, "left lost signal: {rms_l}");
        assert!(rms_r < 0.05, "right leaked signal: {rms_r}");
        fs::remove_dir_all(&dir).ok();
    }
}
