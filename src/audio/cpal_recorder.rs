use super::AudioRecorder;
use crate::types::AudioSamples;
use anyhow::{anyhow, Context, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleFormat, StreamConfig};
use std::sync::{Arc, Mutex};

#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::cast_precision_loss)]
#[allow(clippy::cast_sign_loss)]
struct ResampleState {
    ratio: f64,
    phase: f64,
}

pub struct CpalRecorder {
    device_name: Option<String>,
    samples: Arc<Mutex<Vec<i16>>>,
    stream: Option<cpal::Stream>,
}

impl CpalRecorder {
    #[allow(clippy::missing_errors_doc)]
    pub fn new(device: Option<&str>) -> Result<Self> {
        Ok(CpalRecorder {
            device_name: device.map(ToString::to_string),
            samples: Arc::new(Mutex::new(Vec::new())),
            stream: None,
        })
    }

    fn start(&mut self) -> Result<()> {
        let host = cpal::default_host();

        let device = if let Some(ref name) = self.device_name {
            host.input_devices()
                .context("failed to enumerate devices")?
                .find(|d| d.name().map(|n| n == *name).unwrap_or(false))
                .with_context(|| format!("audio device '{name}' not found"))?
        } else {
            host.default_input_device()
                .context("no default audio input device found")?
        };

        self.device_name = device.name().ok();

        let stream = self.build_stream(&device)?;

        stream.play().context("failed to start audio stream")?;

        self.stream = Some(stream);

        Ok(())
    }

    fn stop(&mut self) -> Result<AudioSamples> {
        self.stream = None;

        let mut samples = self
            .samples
            .lock()
            .map_err(|e| anyhow!("failed to lock samples: {e}"))?;
        let i16_samples = samples.drain(..).collect::<Vec<_>>();

        let f32_samples: Vec<f32> = i16_samples
            .iter()
            .map(|&s| f32::from(s) / 32768.0)
            .collect();

        Ok(AudioSamples(f32_samples))
    }

    fn build_stream(&self, device: &cpal::Device) -> Result<cpal::Stream> {
        let ideal_config = StreamConfig {
            channels: 1,
            sample_rate: cpal::SampleRate(16_000),
            buffer_size: cpal::BufferSize::Default,
        };

        if let Ok(stream) = self.build_direct_i16_stream(device, &ideal_config) {
            return Ok(stream);
        }

        if let Ok(stream) = self.build_direct_f32_stream(device, &ideal_config) {
            return Ok(stream);
        }

        let default_config = device
            .default_input_config()
            .context("failed to get any supported input config from audio device")?;

        let native_rate = default_config.sample_rate().0;
        let native_channels = default_config.channels();
        let native_format = default_config.sample_format();

        let stream_config = StreamConfig {
            channels: native_channels,
            sample_rate: cpal::SampleRate(native_rate),
            buffer_size: cpal::BufferSize::Default,
        };

        match native_format {
            SampleFormat::I16 => self.build_resampling_i16_stream(
                device,
                &stream_config,
                native_rate,
                native_channels,
            ),
            _ => self.build_resampling_f32_stream(
                device,
                &stream_config,
                native_rate,
                native_channels,
            ),
        }
        .context("failed to build audio input stream with any supported configuration")
    }

    fn build_direct_i16_stream(
        &self,
        device: &cpal::Device,
        config: &StreamConfig,
    ) -> Result<cpal::Stream> {
        let samples_arc = Arc::clone(&self.samples);

        let stream = device
            .build_input_stream(
                config,
                move |data: &[i16], _: &cpal::InputCallbackInfo| {
                    if let Ok(mut guard) = samples_arc.try_lock() {
                        guard.extend_from_slice(data);
                    }
                },
                |err| tracing::error!("audio stream error: {err}"),
                None,
            )
            .context("failed to build i16 audio input stream")?;

        Ok(stream)
    }

    fn build_direct_f32_stream(
        &self,
        device: &cpal::Device,
        config: &StreamConfig,
    ) -> Result<cpal::Stream> {
        let samples_arc = Arc::clone(&self.samples);

        let stream = device
            .build_input_stream(
                config,
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    if let Ok(mut guard) = samples_arc.try_lock() {
                        for &s in data {
                            let clamped = s.clamp(-1.0, 1.0);
                            #[allow(clippy::cast_possible_truncation)]
                            {
                                guard.push((clamped * 32767.0) as i16);
                            }
                        }
                    }
                },
                |err| tracing::error!("audio stream error: {err}"),
                None,
            )
            .context("failed to build f32 audio input stream")?;

        Ok(stream)
    }

    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::cast_precision_loss)]
    #[allow(clippy::cast_sign_loss)]
    fn build_resampling_i16_stream(
        &self,
        device: &cpal::Device,
        config: &StreamConfig,
        native_rate: u32,
        native_channels: u16,
    ) -> Result<cpal::Stream> {
        let samples_arc = Arc::clone(&self.samples);
        let state = Arc::new(Mutex::new(ResampleState {
            ratio: f64::from(native_rate) / 16_000.0,
            phase: 0.0,
        }));

        let stream = device
            .build_input_stream(
                config,
                move |data: &[i16], _: &cpal::InputCallbackInfo| {
                    let ch = native_channels as usize;

                    let mono: Vec<f32> = if ch > 1 {
                        data.chunks(ch)
                            .map(|frame| {
                                frame.iter().map(|&s| f32::from(s) / 32768.0).sum::<f32>()
                                    / ch as f32
                            })
                            .collect()
                    } else {
                        data.iter().map(|&s| f32::from(s) / 32768.0).collect()
                    };

                    if let Ok(mut st) = state.lock() {
                        let ratio = st.ratio;
                        let mut phase = st.phase;
                        let len = mono.len() as f64;
                        let mut resampled = Vec::new();

                        while phase < len {
                            let idx = phase as usize;
                            let frac = (phase - idx as f64) as f32;
                            let a = mono[idx];
                            let b = if idx + 1 < mono.len() {
                                mono[idx + 1]
                            } else {
                                a
                            };
                            let sample = a + (b - a) * frac;
                            let clamped = sample.clamp(-1.0, 1.0);
                            resampled.push((clamped * 32767.0) as i16);
                            phase += ratio;
                        }

                        st.phase = phase - len;

                        if let Ok(mut guard) = samples_arc.try_lock() {
                            guard.extend_from_slice(&resampled);
                        }
                    }
                },
                |err| tracing::error!("audio stream error: {err}"),
                None,
            )
            .context("failed to build i16 resampling audio input stream")?;

        Ok(stream)
    }

    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::cast_precision_loss)]
    #[allow(clippy::cast_sign_loss)]
    fn build_resampling_f32_stream(
        &self,
        device: &cpal::Device,
        config: &StreamConfig,
        native_rate: u32,
        native_channels: u16,
    ) -> Result<cpal::Stream> {
        let samples_arc = Arc::clone(&self.samples);
        let state = Arc::new(Mutex::new(ResampleState {
            ratio: f64::from(native_rate) / 16_000.0,
            phase: 0.0,
        }));

        let stream = device
            .build_input_stream(
                config,
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    let ch = native_channels as usize;

                    let mono: Vec<f32> = if ch > 1 {
                        data.chunks(ch)
                            .map(|frame| frame.iter().sum::<f32>() / ch as f32)
                            .collect()
                    } else {
                        data.to_vec()
                    };

                    if let Ok(mut st) = state.lock() {
                        let ratio = st.ratio;
                        let mut phase = st.phase;
                        let len = mono.len() as f64;
                        let mut resampled = Vec::new();

                        while phase < len {
                            let idx = phase as usize;
                            let frac = (phase - idx as f64) as f32;
                            let a = mono[idx];
                            let b = if idx + 1 < mono.len() {
                                mono[idx + 1]
                            } else {
                                a
                            };
                            let sample = a + (b - a) * frac;
                            let clamped = sample.clamp(-1.0, 1.0);
                            resampled.push((clamped * 32767.0) as i16);
                            phase += ratio;
                        }

                        st.phase = phase - len;

                        if let Ok(mut guard) = samples_arc.try_lock() {
                            guard.extend_from_slice(&resampled);
                        }
                    }
                },
                |err| tracing::error!("audio stream error: {err}"),
                None,
            )
            .context("failed to build f32 resampling audio input stream")?;

        Ok(stream)
    }
}

impl AudioRecorder for CpalRecorder {
    fn start(&mut self) -> Result<()> {
        CpalRecorder::start(self)
    }

    fn stop(&mut self) -> Result<AudioSamples> {
        CpalRecorder::stop(self)
    }
}

#[allow(clippy::missing_errors_doc)]
pub fn list_devices() -> Result<Vec<String>> {
    let host = cpal::default_host();
    let devices = host
        .input_devices()
        .context("failed to enumerate input devices")?;
    Ok(devices.filter_map(|d| d.name().ok()).collect())
}
