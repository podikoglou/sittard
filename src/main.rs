use anyhow::Result;

fn main() -> Result<()> {
    std::env::set_var("PIPEWIRE_LOG_LEVEL", "0");
    std::env::set_var("JACK_NO_START_SERVER", "1");
    std::env::set_var("JACK_NO_AUDIO_RESERVATION", "1");

    Ok(())
}
