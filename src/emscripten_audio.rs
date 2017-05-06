extern "C" { fn emscripten_asm_const(code: *const ::std::os::raw::c_char); }

pub struct Audio {
}

#[derive(Debug)]
pub enum Error {
}

impl Audio {
    pub fn new() -> Result<Audio, Error> {
        Ok(Audio {})
    }

    pub fn play_jump(&self) {
        unsafe {
            emscripten_asm_const(b"play_jump()" as *const u8);
        }
    }

    pub fn play_wall(&self, _vol: f32) {
        unsafe {
            emscripten_asm_const(b"play_wall()" as *const u8);
        }
    }
}
