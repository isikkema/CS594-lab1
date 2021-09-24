use crate::{error::RangleError, rangle_display::Color};

pub struct FrameBuffer {
    width: u16,
    height: u16,
    pub buffer: Vec<Color>,
}

impl FrameBuffer {
    pub fn new(width: u16, height: u16) -> Result<Self, RangleError> {
        Ok(FrameBuffer {
            width: width,
            height: height,
            buffer: vec![(0, 0, 0, 0); (width as usize * height as usize).into()],
        })
    }

    pub fn get_size(&self) -> (u16, u16) {
        (self.width, self.height)
    }

    pub fn draw_point(&mut self, x: usize, y: usize, color: Color) {
        let index = y * self.width as usize + x;

        self.buffer[index] = color;
    }

    pub fn fill_buffer(&mut self, val: Color) {
        let _ = self.buffer.iter_mut().map(|v| *v = val).count();
    }
}
