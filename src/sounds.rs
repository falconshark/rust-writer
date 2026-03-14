// sounds.rs - Typewriter sound effects

use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};
use std::io::Cursor;

// Embed CC0 typewriter sounds directly into the binary.
// Source: BigSoundBank (https://bigsoundbank.com), CC0
// Converted to 16-bit 22050 Hz mono to reduce binary size.
static KEY_CLICK_WAV: &[u8] = include_bytes!("../assets/sounds/key_click_final.wav");
static CARRIAGE_RETURN_WAV: &[u8] = include_bytes!("../assets/sounds/carriage_return_small.wav");

pub struct AudioPlayer {
    // Stream must be kept alive for audio output to work
    _stream: OutputStream,
    handle: OutputStreamHandle,
}

impl AudioPlayer {
    pub fn new() -> Option<Self> {
        match OutputStream::try_default() {
            Ok((stream, handle)) => Some(Self { _stream: stream, handle }),
            Err(_) => None,
        }
    }

    pub fn play_click(&self) {
        self.play_wav(KEY_CLICK_WAV);
    }

    pub fn play_return(&self) {
        self.play_wav(CARRIAGE_RETURN_WAV);
    }

    fn play_wav(&self, data: &'static [u8]) {
        let sink = match Sink::try_new(&self.handle) {
            Ok(s) => s,
            Err(_) => return,
        };
        let cursor = Cursor::new(data);
        let decoder = match Decoder::new(cursor) {
            Ok(d) => d,
            Err(_) => return,
        };
        sink.append(decoder);
        sink.detach();
    }
}
