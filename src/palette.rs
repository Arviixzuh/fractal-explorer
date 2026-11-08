use creamui::Color;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Palette {
    MonochromeTerminal,
    CyberpunkNeon,
    DeepVoidBlue,
}

impl Palette {
    pub const ALL: [Self; 3] = [
        Self::MonochromeTerminal,
        Self::CyberpunkNeon,
        Self::DeepVoidBlue,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::MonochromeTerminal => "MONOCHROME TERMINAL",
            Self::CyberpunkNeon => "CYBERPUNK NEON",
            Self::DeepVoidBlue => "DEEP VOID BLUE",
        }
    }

    pub fn color(self, smooth_iteration: f64) -> [u8; 3] {
        let (phase, amplitude, offset) = match self {
            Self::MonochromeTerminal => (0.18, 0.38, [0.20, 0.88, 0.58]),
            Self::CyberpunkNeon => (0.12, 0.58, [0.82, 0.16, 0.68]),
            Self::DeepVoidBlue => (0.08, 0.48, [0.08, 0.38, 0.92]),
        };
        let t = smooth_iteration * phase;
        let channels = [
            offset[0] + amplitude * (t + 0.00).cos(),
            offset[1] + amplitude * (t + 2.09).cos(),
            offset[2] + amplitude * (t + 4.18).cos(),
        ];
        channels.map(|channel| (channel.clamp(0.0, 1.0) * 255.0) as u8)
    }

    pub fn interior(self) -> Color {
        match self {
            Self::MonochromeTerminal => Color::rgb(3, 10, 8),
            Self::CyberpunkNeon => Color::rgb(6, 5, 13),
            Self::DeepVoidBlue => Color::rgb(2, 5, 16),
        }
    }
}
