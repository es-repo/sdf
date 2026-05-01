use crate::lerp;
use pixels::wgpu::Color;

pub trait ColorExt {
    fn to_u8_array(&self) -> [u8; 4];
    fn blend(&self, dst: Color) -> Color;
    fn lerp(&self, other: Color, t: f32) -> Color;
    fn lerp_gray(&self, value: f32, t: f32) -> Color;
    fn with_alpha(&self, alpha: f64) -> Color;
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

    fn blend(&self, dst: Color) -> Color {
        let sa = self.a.clamp(0.0, 1.0);
        let da = dst.a.clamp(0.0, 1.0);

        let out_a = sa + da * (1.0 - sa);

        if out_a == 0.0 {
            return Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 0.0,
            };
        }

        Color {
            r: (self.r * sa + dst.r * da * (1.0 - sa)) / out_a,
            g: (self.g * sa + dst.g * da * (1.0 - sa)) / out_a,
            b: (self.b * sa + dst.b * da * (1.0 - sa)) / out_a,
            a: out_a,
        }
    }

    fn lerp(&self, other: Color, t: f32) -> Color {
        Color {
            r: lerp(self.r, other.r, t as f64),
            g: lerp(self.g, other.g, t as f64),
            b: lerp(self.b, other.b, t as f64),
            a: lerp(self.a, other.a, t as f64),
        }
    }

    fn lerp_gray(&self, value: f32, t: f32) -> Color {
        self.lerp(
            Color {
                r: value as f64,
                g: value as f64,
                b: value as f64,
                a: self.a,
            },
            t,
        )
    }

    fn with_alpha(&self, alpha: f64) -> Color {
        Color {
            r: self.r,
            g: self.g,
            b: self.b,
            a: alpha,
        }
    }
}
