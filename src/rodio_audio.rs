extern crate rodio;

use self::rodio::source::*;
use self::rodio::Decoder;
use std::io;
use std::cell::Cell;
use configuration::CFG;

thread_local! {
    static CURRENT_SND: Cell<&'static str> = Cell::new("none");
}

#[derive(Debug)]
pub enum Error {
    Io(::std::io::Error),
    Decoder(rodio::decoder::DecoderError),
    NoEndpoint,
}
impl From<::std::io::Error> for Error {
    fn from(err: ::std::io::Error) -> Error {
        Error::Io(err)
    }
}
impl From<rodio::decoder::DecoderError> for Error {
    fn from(err: rodio::decoder::DecoderError) -> Error {
        Error::Decoder(err)
    }
}
impl ::std::fmt::Display for Error {
    fn fmt(&self, fmt: &mut ::std::fmt::Formatter) -> Result<(), ::std::fmt::Error> {
        let snd = CURRENT_SND.with(|cell| cell.get());
        use self::Error::*;
        match *self {
            Io(ref e) => write!(fmt, "file {}: io error: {}", snd, e),
            Decoder(ref e) => write!(fmt, "file {}: decode error: {:?}", snd, e),
            NoEndpoint => write!(fmt, "no audio device available"),
        }
    }
}

#[cfg(not(feature = "include_all"))]
type FileType = Vec<u8>;

#[cfg(not(feature = "include_all"))]
fn read_snd_files() -> Result<(Vec<u8>, Vec<u8>, Vec<u8>), Error> {
    use std::fs::File;
    use std::io::Read;
    let mut wall = Vec::new();
    CURRENT_SND.with(|cell| cell.set("sounds/wall.ogg"));
    File::open("sounds/wall.ogg")?.read_to_end(&mut wall)?;
    let mut jump = Vec::new();
    CURRENT_SND.with(|cell| cell.set("sounds/jump.ogg"));
    File::open("sounds/jump.ogg")?.read_to_end(&mut jump)?;
    let mut gong = Vec::new();
    CURRENT_SND.with(|cell| cell.set("sounds/gong.ogg"));
    File::open("sounds/gong.ogg")?.read_to_end(&mut gong)?;
    Ok((wall, jump, gong))
}

#[cfg(feature = "include_all")]
type FileType = &'static [u8];

#[cfg(feature = "include_all")]
fn read_snd_files() -> Result<(&'static [u8], &'static [u8], &'static [u8]), Error> {
    let wall = include_bytes!("sounds/wall.ogg");
    let jump = include_bytes!("sounds/jump.ogg");
    let gong = include_bytes!("sounds/gong.ogg");
    Ok((wall, jump, gong))
}

pub struct Audio {
    endpoint: rodio::Endpoint,
    wall: Buffered<Amplify<Decoder<io::Cursor<FileType>>>>,
    jump: Buffered<Amplify<Decoder<io::Cursor<FileType>>>>,
    // gong: Buffered<Amplify<Decoder<io::Cursor<FileType>>>>,
}

impl Audio {
    pub fn new() -> Result<Audio, Error> {
        let snds = read_snd_files()?;
        Ok(Audio {
            endpoint: rodio::get_default_endpoint().ok_or(Error::NoEndpoint)?,
            wall: Decoder::new(io::Cursor::new(snds.0))?
                .amplify(CFG.audio.wall_volume)
                .buffered(),
            jump: Decoder::new(io::Cursor::new(snds.1))?
                .amplify(CFG.audio.jump_volume)
                .buffered(),
            // gong: Decoder::new(io::Cursor::new(snds.2))?
            //     .amplify(CFG.audio.gong_volume)
            //     .buffered(),
        })
    }

    pub fn play_jump(&self) {
        let sink = rodio::Sink::new(&self.endpoint);
        sink.append(self.jump.clone());
        sink.detach();
    }

    pub fn play_wall(&self, vol: f32) {
        if vol > 0. {
            let sink = rodio::Sink::new(&self.endpoint);
            sink.append(self.wall.clone().amplify(vol));
            sink.detach();
        }
    }

    // pub fn play_gong(&self) {
    //     let sink = rodio::Sink::new(&self.endpoint);
    //     sink.append(self.gong.clone());
    //     sink.detach();
    // }
}
