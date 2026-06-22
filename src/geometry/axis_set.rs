#[derive(Clone, Copy, PartialEq, Eq, Debug, serde::Serialize, serde::Deserialize)]
pub enum AxisSet {
    X,
    Y,
    Z,
    XY,
    XZ,
    YZ,
    XYZ,
}

impl AxisSet {
    pub const ALL: [Self; 7] = [Self::X, Self::Y, Self::Z, Self::XY, Self::XZ, Self::YZ, Self::XYZ];

    pub fn label(self) -> &'static str {
        match self {
            Self::X => "X",
            Self::Y => "Y",
            Self::Z => "Z",
            Self::XY => "XY",
            Self::XZ => "XZ",
            Self::YZ => "YZ",
            Self::XYZ => "XYZ",
        }
    }

    pub fn has_x(self) -> bool {
        matches!(self, Self::X | Self::XY | Self::XZ | Self::XYZ)
    }

    pub fn has_y(self) -> bool {
        matches!(self, Self::Y | Self::XY | Self::YZ | Self::XYZ)
    }

    pub fn has_z(self) -> bool {
        matches!(self, Self::Z | Self::XZ | Self::YZ | Self::XYZ)
    }
}
