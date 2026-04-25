#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ColorSwatchStyle {
    Circle,
    Ring,
    Box,
}

impl ColorSwatchStyle {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "circle" => Some(Self::Circle),
            "ring"   => Some(Self::Ring),
            "box"    => Some(Self::Box),
            _ => None,
        }
    }
}

impl Default for ColorSwatchStyle {
    fn default() -> Self { Self::Circle }
}

pub fn colors(style: ColorSwatchStyle) -> String {
    let (swatch, reset) = match style {
        ColorSwatchStyle::Circle => ("●", "\x1b[0m"),
        ColorSwatchStyle::Ring   => ("◉", "\x1b[0m"),
        ColorSwatchStyle::Box    => ("█", "\x1b[0m"),
    };

    let mut result = String::new();

    // Terminal palette colors 0–7 via 256-color mode — reads actual theme colors
    for idx in 0u8..8 {
        result.push_str(&format!("\x1b[38;5;{}m{}", idx, swatch));
        result.push_str(reset);
        result.push(' ');
    }
    if result.ends_with(' ') { result.pop(); }

    result
}