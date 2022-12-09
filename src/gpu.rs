use sdl2::{
    Sdl,
    video::Window,
    render::Canvas,
    rect::Rect
};

pub struct Gpu {
    canvas: Canvas<Window>,
    scale: u32
}

impl Gpu {
    pub fn new (ctx: &Sdl, scale: u32) -> Self {
        let video = ctx.video().unwrap();
        let window = video.window("test", 64 * scale, 32 * scale)
            .position_centered()
            .build()
            .expect("window error");
        let canvas = window.into_canvas().build().expect("canvas error");
        return Gpu {
            canvas: canvas,
            scale: scale
        }
    }

    pub fn draw(&mut self, vram: &[[u8; 64]; 32]) {
        let texture_creator = self.canvas.texture_creator();
        let mut texture = texture_creator.create_texture_streaming(None, 64, 32).unwrap();
        texture.with_lock(None, |buffer: &mut [u8], pitch: usize| {
            for y in 0..32 {
                for x in 0..64 {
                    let pixel = vram[y][x];
                    let offset = y * pitch + x * 3;
                    let (r, g, b) = if pixel > 0 {(255,255,255)} else {(0,0,0)};
                    buffer[offset] = r;
                    buffer[offset + 1] = g;
                    buffer[offset + 2] = b;
                }
            }
        }).unwrap();
        self.canvas.copy(&texture, None, Some(Rect::new(0, 0, 64 * self.scale, 32 * self.scale))).unwrap();
    }
}

