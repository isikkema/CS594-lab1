use crossterm::{
    cursor,
    style::{self, style},
    terminal, ExecutableCommand, QueueableCommand,
};
use std::io::{stdout, Stdout, Write};

use crate::{error::RangleError, rangle_display::{Color, RangleDisplay}};


pub struct AsciiDisplay {
    pub stdout: Stdout,
    width: u16,
    height: u16,
    background_color: (f32, f32, f32, f32),
}

impl RangleDisplay for AsciiDisplay {
    fn new() -> Result<Self, RangleError> {
        terminal::enable_raw_mode()?;

        let (w, h) = terminal::size()?;
        let (w, h) = ((w) / 2, h);

        let mut stdout = stdout();
        stdout
            .queue(terminal::EnterAlternateScreen)?
            .queue(terminal::Clear(terminal::ClearType::All))?
            .queue(cursor::DisableBlinking)?;

        Ok(AsciiDisplay {
            stdout: stdout,
            width: w,
            height: h,
            buffer: vec![(0, 0, 0, 0); (w as usize * h as usize).into()],
            background_color: (0.0, 0.0, 0.0, 1.0),
        })
    }

    fn get_size(&self) -> (u16, u16) {
        (self.width, self.height)
    }

    fn get_background_color(&self) -> Color {
        let (r, b, g, a) = self.background_color;
        
        (
            (r.max(0.0) * 255.0).round() as u8,
            (g.max(0.0) * 255.0).round() as u8,
            (b.max(0.0) * 255.0).round() as u8,
            (a.max(0.0) * 255.0).round() as u8,
        )
    }

    fn set_background_color(&mut self, color: Color) {
        self.background_color = (
            color.0 as f32 / 255.0,
            color.1 as f32 / 255.0,
            color.2 as f32 / 255.0,
            color.3 as f32 / 255.0,
        )
    }

    fn draw_buffer(&mut self) -> Result<(), RangleError> {
        let mut color;

        self.stdout.queue(cursor::MoveTo(0, 0))?;
        for i in 0..self.height as usize {
            for j in 0..self.width as usize {
                color = self.buffer[self.width as usize * i + j];
                if color.3 == 0 {
                    let (r, g, b, a) = self.background_color;
                    color = (
                        (r.max(0.0) * 255.0).round() as u8,
                        (g.max(0.0) * 255.0).round() as u8,
                        (b.max(0.0) * 255.0).round() as u8,
                        (a.max(0.0) * 255.0).round() as u8,
                    );
                }

                self.stdout
                    .queue(cursor::MoveTo((j * 2) as u16, i as u16))?;
                self.stdout.queue(style::PrintStyledContent(
                    style("██").with(style::Color::from((color.0, color.1, color.2))),
                ))?;
            }
        }

        self.stdout.flush()?;

        Ok(())
    }
}

impl Drop for AsciiDisplay {
    fn drop(&mut self) {
        let _ = self.stdout.execute(terminal::LeaveAlternateScreen);
        let _ = terminal::disable_raw_mode();
    }
}
