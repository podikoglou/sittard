pub struct AudioSamples(pub Vec<f32>);

#[derive(Clone, Copy)]
pub enum InputEvent {
    KeyDown,
    KeyUp,
}

pub enum AppEvent {
    Input(InputEvent),
    Shutdown,
    TranscriptionComplete(String),
}

pub enum AppState {
    Idle,
    Recording,
    Transcribing,
}
