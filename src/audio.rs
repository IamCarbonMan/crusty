use sdl2::{
    Sdl,
    audio::{AudioCallback, AudioSpecDesired, AudioDevice}
};

pub struct SquareWave {
    phase_inc: f32,
    phase: f32,
    volume: f32
}

impl AudioCallback for SquareWave {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        for x in out.iter_mut() {
            *x = if self.phase <= 0.5 {
                self.volume
            } else {
                -self.volume
            };
            self.phase = (self.phase + self.phase_inc) % 1.0;
        }
    }
}

const DESIRED_SPEC: AudioSpecDesired = AudioSpecDesired {
    freq: Some(44100),
    channels: Some(1),
    samples: None
};

pub fn get_audio(ctx: &Sdl) -> AudioDevice<SquareWave> {
    let audio = ctx.audio().unwrap();
    return audio.open_playback(None, &DESIRED_SPEC, |spec| {
        SquareWave {
            phase_inc: 440.0 / spec.freq as f32,
            phase: 0.0,
            volume: 0.25
        }
    }).unwrap();
}