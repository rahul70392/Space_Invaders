use std::error::Error;
use rusty_audio::Audio;

fn main() -> Result <(), Box<dyn Error>> {
    let mut audio = Audio::new();

    for item in &["explode", "lose", "move", "pew", "startup", "win"] {
        audio.add(item, &format!("sounds/{}.wav", item));
    }
    audio.play("startup");

    //cleanup
    audio.wait();
    Ok(())
}
