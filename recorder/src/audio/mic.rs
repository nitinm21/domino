use anyhow::{Context, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use ringbuf::traits::Producer;
use ringbuf::HeapProd;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

pub struct MicCapture {
    pub stream: cpal::Stream,
    pub dropped_samples: Arc<AtomicU64>,
}

pub fn start_mic_capture(mut producer: HeapProd<f32>) -> Result<MicCapture> {
    let host = cpal::default_host();
    let device = host
        .default_input_device()
        .context("no default input device found")?;

    tracing::info!(
        device = device.name().unwrap_or_default(),
        "using input device"
    );

    let config = cpal::StreamConfig {
        channels: 1,
        sample_rate: cpal::SampleRate(48000),
        buffer_size: cpal::BufferSize::Default,
    };

    let dropped_samples = Arc::new(AtomicU64::new(0));
    let dropped = Arc::clone(&dropped_samples);

    let stream = device
        .build_input_stream(
            &config,
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                let pushed = producer.push_slice(data);
                if pushed < data.len() {
                    dropped.fetch_add((data.len() - pushed) as u64, Ordering::Relaxed);
                }
            },
            move |err| {
                tracing::error!(%err, "mic capture stream error");
            },
            None,
        )
        .context("failed to build mic input stream (microphone permission may be denied)")?;

    stream.play().context("failed to start mic capture")?;

    Ok(MicCapture {
        stream,
        dropped_samples,
    })
}
