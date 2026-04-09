use pixels::wgpu::Color;

pub trait ColorExt {
    fn to_u8_array(&self) -> [u8; 4];
    fn blend(&mut self, dst: Color) -> &mut Self;
}

impl ColorExt for Color {
    fn to_u8_array(&self) -> [u8; 4] {
        [
            (self.r * 255.0) as u8,
            (self.g * 255.0) as u8,
            (self.b * 255.0) as u8,
            (self.a * 255.0) as u8,
        ]
    }

    fn blend(&mut self, dst: Color) -> &mut Self {
        let sa = self.a.clamp(0.0, 1.0);
        let da = dst.a.clamp(0.0, 1.0);

        let out_a = sa + da * (1.0 - sa);

        if out_a == 0.0 {
            self.r = 0.0;
            self.g = 0.0;
            self.b = 0.0;
            self.a = 0.0;
        } else {
            self.r = (self.r * sa + dst.r * da * (1.0 - sa)) / out_a;
            self.g = (self.g * sa + dst.g * da * (1.0 - sa)) / out_a;
            self.b = (self.b * sa + dst.b * da * (1.0 - sa)) / out_a;
            self.a = out_a;
        }

        self
    }
}
