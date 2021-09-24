use rangle::{FrameBuffer, error::RangleError, rangle_display::{Color, RangleDisplay}};


pub struct JgraphDisplay {
    width: u16,
    height: u16,
    background_color: (f32, f32, f32, f32),
}

impl JgraphDisplay {
    pub fn new(width: u16, height: u16, color: (f32, f32, f32, f32)) -> Result<Self, RangleError> {
        Ok(JgraphDisplay {
            width: width,
            height: height,
            background_color: color,
        })
    }
}

impl RangleDisplay for JgraphDisplay {
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

    fn draw_buffer(&self, frame_buffer: &FrameBuffer) -> Result<(), RangleError> {
        let mut color;

        let (w, h) = self.get_size();
        
        let (xsize_ps, ysize_ps) = (w as f32, h as f32);
        let (xsize_in, ysize_in) = (xsize_ps / 72.0, ysize_ps / 72.0);

        let paper_size = (612.0, 792.0);
        let xtranslate = (paper_size.0 - xsize_ps) / 4.0;
        let ytranslate = ((paper_size.1 - ysize_ps) / 4.0).round();

        println!("bbox {} {} {} {}", xtranslate, ytranslate, xsize_ps+xtranslate, ysize_ps+ytranslate);
        println!("newgraph");
        println!("xaxis nodraw min 0 max {} size {}", w, xsize_in);
        println!("yaxis nodraw min 0 max {} size {}", h, ysize_in);
        for i in 0..self.height as usize {
            for j in 0..self.width as usize {
                color = frame_buffer.buffer[self.width as usize * i + j];
                if color.3 == 0 {
                    let (r, g, b, a) = self.background_color;
                    color = (
                        (r.max(0.0) * 255.0).round() as u8,
                        (g.max(0.0) * 255.0).round() as u8,
                        (b.max(0.0) * 255.0).round() as u8,
                        (a.max(0.0) * 255.0).round() as u8,
                    );
                }
                
                let (r, g, b) = (
                    color.0 as f32 / 255.0,
                    color.1 as f32 / 255.0,
                    color.2 as f32 / 255.0,
                );

                print!("newcurve marktype box marksize 1 1 ");
                print!("color {} {} {} ", r, g, b);
                print!("pts {} {} ", j, self.height as usize - i);
            }
        }

        Ok(())
    }
}
