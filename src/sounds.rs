// sounds.rs - Typewriter sound effects

use rodio::{Decoder, OutputStream, Sink};
use std::io::Cursor;
use std::sync::mpsc;

// Embed FocusWriter typewriter sounds directly into the binary.
// Source: FocusWriter by Graeme Gott (https://github.com/gottcode/focuswriter), GPL-3.0
// Converted from stereo 44100 Hz to 16-bit 22050 Hz mono to reduce binary size.
static KEY_CLICK_WAV: &[u8] = include_bytes!("../assets/sounds/keyany_final.wav");
static CARRIAGE_RETURN_WAV: &[u8] = include_bytes!("../assets/sounds/keyenter_final.wav");

enum AudioCommand {
    Click,
    Return,
}

/// Handle to a background audio thread. Cheap to clone-send commands to.
/// The background thread owns the OutputStream (which is not Send), so it
/// never crosses thread boundaries. Startup is non-blocking: the thread
/// initialises the audio device while the UI is already visible.
pub struct AudioPlayer {
    sender: mpsc::SyncSender<AudioCommand>,
}

impl AudioPlayer {
    /// Spawns the audio thread and returns immediately — does not block the UI.
    pub fn new() -> Option<Self> {
        let (tx, rx) = mpsc::sync_channel::<AudioCommand>(8);
        std::thread::spawn(move || {
            let (_stream, handle) = match OutputStream::try_default() {
                Ok(v) => v,
                Err(_) => return,
            };
            for cmd in rx {
                let data: &'static [u8] = match cmd {
                    AudioCommand::Click => KEY_CLICK_WAV,
                    AudioCommand::Return => CARRIAGE_RETURN_WAV,
                };
                let sink = match Sink::try_new(&handle) {
                    Ok(s) => s,
                    Err(_) => continue,
                };
                if let Ok(decoder) = Decoder::new(Cursor::new(data)) {
                    sink.append(decoder);
                    sink.detach();
                }
            }
        });
        Some(Self { sender: tx })
    }

    pub fn play_click(&self) {
        let _ = self.sender.try_send(AudioCommand::Click);
    }

    pub fn play_return(&self) {
        let _ = self.sender.try_send(AudioCommand::Return);
    }
}
