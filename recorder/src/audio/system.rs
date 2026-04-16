use anyhow::{Context, Result};
use ringbuf::traits::Producer;
use ringbuf::HeapProd;
use screencapturekit::cm::{AudioBufferList, CMSampleBuffer};
use screencapturekit::prelude::*;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

pub struct SystemCapture {
    pub stream: SCStream,
    pub dropped_samples: Arc<AtomicU64>,
}

impl SystemCapture {
    pub fn stop(self) {
        if let Err(e) = self.stream.stop_capture() {
            tracing::warn!(?e, "SCStream stop_capture failed");
        }
    }
}

struct AudioHandler {
    producer: Mutex<HeapProd<f32>>,
    dropped_samples: Arc<AtomicU64>,
    scratch: Mutex<Vec<f32>>,
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
        // Planar: each buffer is one channel, sample-aligned across buffers.
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
        // Packed/interleaved (or mono): first buffer holds everything.
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

    // Audio-only is not supported by SCK — we attach a 2x2 video stream
    // and ignore the screen frames.
    let config = SCStreamConfiguration::new()
        .with_width(2)
        .with_height(2)
        .with_captures_audio(true)
        .with_excludes_current_process_audio(true)
        .with_sample_rate(48000)
        .with_channel_count(2);

    let dropped_samples = Arc::new(AtomicU64::new(0));
    let handler = AudioHandler {
        producer: Mutex::new(producer),
        dropped_samples: Arc::clone(&dropped_samples),
        scratch: Mutex::new(Vec::with_capacity(4096)),
    };

    let mut stream = SCStream::new(&filter, &config);
    stream.add_output_handler(handler, SCStreamOutputType::Audio);
    stream
        .start_capture()
        .context("SCStream start_capture failed")?;

    Ok(SystemCapture {
        stream,
        dropped_samples,
    })
}
