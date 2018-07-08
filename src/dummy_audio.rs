pub struct Error;

pub struct Audio;

impl Audio {
    pub fn new() -> Result<Audio, Error> {
        Ok(Audio)
    }

    pub fn play_jump(&self) {
    }

    pub fn play_wall(&self, _vol: f32) {
    }
}
