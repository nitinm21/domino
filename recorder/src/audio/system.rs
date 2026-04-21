use anyhow::{Context, Result};
use ringbuf::traits::Producer;
use ringbuf::HeapProd;
use screencapturekit::cm::{AudioBufferList, CMSampleBuffer};
use screencapturekit::error::SCError;
use screencapturekit::prelude::*;
use screencapturekit::stream::delegate_trait::SCStreamDelegateTrait;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

pub struct SystemCapture {
    pub dropped_samples: Arc<AtomicU64>,
    shutdown: Arc<AtomicBool>,
    supervisor: Option<JoinHandle<()>>,
    stream: Arc<Mutex<Option<SCStream>>>,
}

impl SystemCapture {
    pub fn stop(mut self) {
        self.shutdown.store(true, Ordering::SeqCst);

        if let Some(stream) = self.stream.lock().ok().and_then(|mut g| g.take()) {
            if let Err(e) = stream.stop_capture() {
                tracing::warn!(?e, "SCStream stop_capture failed");
            }
        }

        if let Some(handle) = self.supervisor.take() {
            let _ = handle.join();
        }
    }
}

#[derive(Clone)]
struct AudioHandler {
    producer: Arc<Mutex<HeapProd<f32>>>,
    dropped_samples: Arc<AtomicU64>,
    scratch: Arc<Mutex<Vec<f32>>>,
}

impl SCStreamOutputTrait for AudioHandler {
    fn did_output_sample_buffer(&self, sample: CMSampleBuffer, output_type: SCStreamOutputType) {
        if !matches!(output_type, SCStreamOutputType::Audio) {
            return;
        }
        let Some(list) = sample.audio_buffer_list() else {
            return;
        };

        let Ok(mut scratch) = self.scratch.lock() else {
            return;
        };
        scratch.clear();
        downmix_to_mono(&list, &mut scratch);
        if scratch.is_empty() {
            return;
        }

        let Ok(mut producer) = self.producer.lock() else {
            return;
        };
        let pushed = producer.push_slice(&scratch);
        if pushed < scratch.len() {
            self.dropped_samples
                .fetch_add((scratch.len() - pushed) as u64, Ordering::Relaxed);
        }
    }
}

/// Delegate that flips `died` when replayd tells us the stream stopped.
/// macOS (via replayd) proactively kills SCStreams under memory pressure —
/// see `handleMemoryWarningForCurrentActiveSession` / `stopAllStreamsWithError`
/// in replayd's logs. We use this flag to trigger a supervised restart.
struct DeathWatcher {
    died: Arc<AtomicBool>,
}

impl SCStreamDelegateTrait for DeathWatcher {
    fn did_stop_with_error(&self, error: SCError) {
        tracing::warn!(%error, "SCStream died — will restart");
        self.died.store(true, Ordering::SeqCst);
    }

    fn stream_did_stop(&self, error: Option<String>) {
        if let Some(e) = error {
            tracing::warn!(error = %e, "SCStream stopped with error — will restart");
            self.died.store(true, Ordering::SeqCst);
        }
    }
}

/// Down-mix a CoreAudio AudioBufferList of f32 PCM into mono samples
/// appended to `out`. Handles both planar (one buffer per channel,
/// each buffer's `number_channels == 1`) and packed/interleaved
/// (single buffer with multiple channels) layouts.
fn downmix_to_mono(list: &AudioBufferList, out: &mut Vec<f32>) {
    let buffers: Vec<&screencapturekit::cm::AudioBuffer> = list.iter().collect();
    if buffers.is_empty() {
        return;
    }

    let all_planar = buffers.iter().all(|b| b.number_channels == 1);

    if all_planar && buffers.len() > 1 {
        let n_samples = buffers[0].data().len() / std::mem::size_of::<f32>();
        let n_channels = buffers.len() as f32;
        out.reserve(n_samples);
        for i in 0..n_samples {
            let mut sum = 0.0f32;
            for buf in &buffers {
                if let Some(s) = read_f32(buf.data(), i) {
                    sum += s;
                }
            }
            out.push(sum / n_channels);
        }
    } else {
        let buf = buffers[0];
        let channels = buf.number_channels.max(1) as usize;
        let total = buf.data().len() / std::mem::size_of::<f32>();
        let frames = total / channels;
        out.reserve(frames);
        for i in 0..frames {
            let mut sum = 0.0f32;
            for ch in 0..channels {
                if let Some(s) = read_f32(buf.data(), i * channels + ch) {
                    sum += s;
                }
            }
            out.push(sum / channels as f32);
        }
    }
}

fn read_f32(bytes: &[u8], index: usize) -> Option<f32> {
    let start = index * std::mem::size_of::<f32>();
    let end = start + std::mem::size_of::<f32>();
    if end > bytes.len() {
        return None;
    }
    Some(f32::from_le_bytes([
        bytes[start],
        bytes[start + 1],
        bytes[start + 2],
        bytes[start + 3],
    ]))
}

fn build_config() -> SCStreamConfiguration {
    // Audio-only is not supported by SCK — we attach a minimal video stream
    // and ignore the frames. 2x2 @ 1fps with a shallow queue keeps replayd's
    // working set tiny, so it's less likely to be reaped under memory pressure.
    SCStreamConfiguration::new()
        .with_width(2)
        .with_height(2)
        .with_fps(1)
        .with_queue_depth(3)
        .with_captures_audio(true)
        .with_excludes_current_process_audio(true)
        .with_sample_rate(48000)
        .with_channel_count(2)
}

fn build_stream(
    filter: &SCContentFilter,
    handler: AudioHandler,
    died: &Arc<AtomicBool>,
) -> Result<SCStream> {
    let config = build_config();
    let delegate = DeathWatcher {
        died: Arc::clone(died),
    };
    let mut stream = SCStream::new_with_delegate(filter, &config, delegate);
    stream.add_output_handler(handler, SCStreamOutputType::Audio);
    stream
        .start_capture()
        .context("SCStream start_capture failed")?;
    Ok(stream)
}

pub fn start_system_capture(producer: HeapProd<f32>) -> Result<SystemCapture> {
    let content = SCShareableContent::get()
        .context("SCShareableContent::get failed (Screen Recording permission may be denied)")?;

    let display = content
        .displays()
        .into_iter()
        .next()
        .context("no displays available for system audio capture")?;

    tracing::info!("starting system audio capture via ScreenCaptureKit");

    let filter = SCContentFilter::create()
        .with_display(&display)
        .with_excluding_windows(&[])
        .build();

    let dropped_samples = Arc::new(AtomicU64::new(0));
    let died = Arc::new(AtomicBool::new(false));
    let shutdown = Arc::new(AtomicBool::new(false));
    let producer = Arc::new(Mutex::new(producer));
    let scratch = Arc::new(Mutex::new(Vec::with_capacity(4096)));

    let handler = AudioHandler {
        producer: Arc::clone(&producer),
        dropped_samples: Arc::clone(&dropped_samples),
        scratch: Arc::clone(&scratch),
    };

    let first_stream = build_stream(&filter, handler.clone(), &died)?;
    let stream_slot: Arc<Mutex<Option<SCStream>>> = Arc::new(Mutex::new(Some(first_stream)));

    // Supervisor: watches `died`, rebuilds the SCStream if replayd reaps it.
    let supervisor = {
        let shutdown = Arc::clone(&shutdown);
        let died = Arc::clone(&died);
        let stream_slot = Arc::clone(&stream_slot);
        let dropped_samples = Arc::clone(&dropped_samples);
        let handler = handler.clone();
        std::thread::Builder::new()
            .name("system-audio-supervisor".into())
            .spawn(move || {
                supervisor_loop(
                    shutdown,
                    died,
                    stream_slot,
                    filter,
                    handler,
                    dropped_samples,
                );
            })
            .context("spawn system-audio supervisor thread failed")?
    };

    Ok(SystemCapture {
        dropped_samples,
        shutdown,
        supervisor: Some(supervisor),
        stream: stream_slot,
    })
}

fn supervisor_loop(
    shutdown: Arc<AtomicBool>,
    died: Arc<AtomicBool>,
    stream_slot: Arc<Mutex<Option<SCStream>>>,
    filter: SCContentFilter,
    handler: AudioHandler,
    dropped_samples: Arc<AtomicU64>,
) {
    const POLL_INTERVAL: Duration = Duration::from_millis(200);
    const BACKOFF_INITIAL: Duration = Duration::from_millis(250);
    const BACKOFF_MAX: Duration = Duration::from_secs(5);

    let mut restart_count: u64 = 0;

    while !shutdown.load(Ordering::SeqCst) {
        std::thread::sleep(POLL_INTERVAL);
        if !died.swap(false, Ordering::SeqCst) {
            continue;
        }
        if shutdown.load(Ordering::SeqCst) {
            break;
        }

        restart_count += 1;
        let pre_drop = dropped_samples.load(Ordering::Relaxed);
        tracing::warn!(
            restart = restart_count,
            dropped_so_far = pre_drop,
            "system audio stream died — attempting restart"
        );

        // Drop the dead stream so replayd releases its resources before we
        // ask it for a fresh one.
        if let Ok(mut slot) = stream_slot.lock() {
            if let Some(old) = slot.take() {
                let _ = old.stop_capture();
                drop(old);
            }
        }

        let mut backoff = BACKOFF_INITIAL;
        let started_at = Instant::now();
        loop {
            if shutdown.load(Ordering::SeqCst) {
                return;
            }
            match build_stream(&filter, handler.clone(), &died) {
                Ok(new_stream) => {
                    if let Ok(mut slot) = stream_slot.lock() {
                        *slot = Some(new_stream);
                    }
                    // Clear any death signal that fired during restart — we
                    // want the next genuine failure to trigger another cycle.
                    died.store(false, Ordering::SeqCst);
                    tracing::info!(
                        restart = restart_count,
                        downtime_ms = started_at.elapsed().as_millis() as u64,
                        "system audio stream restarted"
                    );
                    break;
                }
                Err(e) => {
                    tracing::warn!(
                        ?e,
                        backoff_ms = backoff.as_millis() as u64,
                        "system audio restart failed — retrying"
                    );
                    std::thread::sleep(backoff);
                    backoff = (backoff * 2).min(BACKOFF_MAX);
                }
            }
        }
    }
}
