use sdl2::{
    Sdl, EventPump,
    keyboard::Scancode
};

pub struct Input {
    events: EventPump
}

impl Input {
    pub fn new(ctx: &Sdl) -> Self {
        Input {
            events: ctx.event_pump().unwrap()
        }
    }

    pub fn poll(&mut self) -> [bool; 16] {
        let keys = [Scancode::X, Scancode::Num1, Scancode::Num2, Scancode::Num3, Scancode::Q, Scancode::W, Scancode::E, Scancode::A, Scancode::S, Scancode::D, Scancode::Z, Scancode::C, Scancode::Num4, Scancode::R, Scancode::F, Scancode::V];
        keys.map(|key| {
            self.events.keyboard_state().is_scancode_pressed(key)
        })
    }
}