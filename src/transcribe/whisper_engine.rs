use super::Transcriber;
use crate::types::AudioSamples;
use anyhow::{anyhow, Result};
use std::path::Path;
use std::sync::mpsc;

type TranscribeRequest = (AudioSamples, mpsc::Sender<Result<String>>);

pub struct WhisperEngine {
    tx: mpsc::Sender<TranscribeRequest>,
}

impl WhisperEngine {
    #[allow(clippy::missing_errors_doc)]
    pub fn new(model_path: &Path, threads: usize) -> Result<Self> {
        let model_str = model_path
            .to_str()
            .ok_or_else(|| anyhow!("model path contains invalid UTF-8"))?
            .to_string();

        let (tx, rx) = mpsc::channel::<TranscribeRequest>();

        std::thread::spawn(move || {
            worker_loop(&model_str, threads, &rx);
        });

        Ok(Self { tx })
    }
}

fn suppress_whisper_logging() {
    unsafe {
        unsafe extern "C" fn noop_log(
            _level: whisper_rs::whisper_rs_sys::ggml_log_level,
            _text: *const std::ffi::c_char,
            _user_data: *mut std::ffi::c_void,
        ) {
        }
        whisper_rs::whisper_rs_sys::whisper_log_set(Some(noop_log), std::ptr::null_mut());
        whisper_rs::whisper_rs_sys::ggml_log_set(Some(noop_log), std::ptr::null_mut());
    }
}

fn worker_loop(model_path: &str, threads: usize, rx: &mpsc::Receiver<TranscribeRequest>) {
    suppress_whisper_logging();

    let ctx = match whisper_rs::WhisperContext::new_with_params(
        model_path,
        whisper_rs::WhisperContextParameters::default(),
    ) {
        Ok(ctx) => ctx,
        Err(e) => {
            tracing::error!("failed to load whisper model: {:?}", e);
            return;
        }
    };

    tracing::info!("whisper model loaded");

    while let Ok((samples, reply)) = rx.recv() {
        let result = if samples.0.len() < 1600 {
            Ok(String::new())
        } else {
            transcribe_inner(&ctx, &samples.0, threads)
        };

        let _ = reply.send(result);
    }
}

fn transcribe_inner(
    ctx: &whisper_rs::WhisperContext,
    audio: &[f32],
    threads: usize,
) -> Result<String> {
    let mut params =
        whisper_rs::FullParams::new(whisper_rs::SamplingStrategy::Greedy { best_of: 1 });
    params.set_n_threads(i32::try_from(threads).unwrap_or(1));
    params.set_language(Some("en"));
    params.set_no_timestamps(true);
    params.set_single_segment(true);
    params.set_suppress_blank(true);
    params.set_print_special(false);
    params.set_print_progress(false);
    params.set_print_realtime(false);
    params.set_print_timestamps(false);

    let mut state = ctx
        .create_state()
        .map_err(|e| anyhow!("failed to create whisper state: {e:?}"))?;

    state
        .full(params, audio)
        .map_err(|e| anyhow!("whisper transcription failed: {e:?}"))?;

    let num_segments = state
        .full_n_segments()
        .map_err(|e| anyhow!("failed to get segment count: {e:?}"))?;

    let mut text = String::new();
    for i in 0..num_segments {
        if let Ok(segment) = state.full_get_segment_text(i) {
            if !text.is_empty() {
                text.push(' ');
            }
            text.push_str(&segment);
        }
    }

    Ok(text.trim().to_string())
}

impl Transcriber for WhisperEngine {
    fn transcribe(&self, samples: AudioSamples) -> Result<String> {
        let (reply_tx, reply_rx) = mpsc::channel();

        self.tx
            .send((samples, reply_tx))
            .map_err(|_| anyhow!("whisper worker thread died"))?;

        reply_rx
            .recv()
            .map_err(|_| anyhow!("whisper worker thread died"))?
    }
}
