use crate::{FrameBuffer, error::RangleError};


pub type Color = (u8, u8, u8, u8);


pub trait RangleDisplay {
    fn get_size(&self) -> (u16, u16);

    fn get_background_color(&self) -> Color;

    fn set_background_color(&mut self, color: Color);

    fn draw_buffer(&self, frame_buffer: &FrameBuffer) -> Result<(), RangleError>;
}
