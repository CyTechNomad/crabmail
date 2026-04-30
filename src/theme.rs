use ratatui::style::Color;

#[derive(Debug, Clone)]
pub struct Theme {
    pub accent: Color,
    pub dimmed: Color,
    pub text: Color,
    pub success: Color,
    pub error: Color,
    pub search: Color,
    pub bar_bg: Color,
    pub mode_fg: Color,
}

impl Theme {
    pub fn from_name(name: &str) -> Self {
        match name {
            "dracula" => Self::dracula(),
            "gruvbox" => Self::gruvbox(),
            "nord" => Self::nord(),
            "solarized" => Self::solarized(),
            _ => Self::default(),
        }
    }

    pub fn available() -> &'static [&'static str] {
        &["default", "dracula", "gruvbox", "nord", "solarized"]
    }

    fn dracula() -> Self {
        Self {
            accent: Color::Rgb(189, 147, 249),   // purple
            dimmed: Color::Rgb(98, 114, 164),     // comment
            text: Color::Rgb(248, 248, 242),      // foreground
            success: Color::Rgb(80, 250, 123),    // green
            error: Color::Rgb(255, 85, 85),       // red
            search: Color::Rgb(241, 250, 140),    // yellow
            bar_bg: Color::Rgb(68, 71, 90),       // current line
            mode_fg: Color::Rgb(40, 42, 54),      // background
        }
    }

    fn gruvbox() -> Self {
        Self {
            accent: Color::Rgb(215, 153, 33),     // yellow
            dimmed: Color::Rgb(146, 131, 116),    // gray
            text: Color::Rgb(235, 219, 178),      // fg
            success: Color::Rgb(152, 151, 26),    // green
            error: Color::Rgb(204, 36, 29),       // red
            search: Color::Rgb(69, 133, 136),     // aqua
            bar_bg: Color::Rgb(60, 56, 54),       // bg1
            mode_fg: Color::Rgb(40, 40, 40),      // bg0
        }
    }

    fn nord() -> Self {
        Self {
            accent: Color::Rgb(136, 192, 208),    // frost
            dimmed: Color::Rgb(76, 86, 106),      // nord3
            text: Color::Rgb(216, 222, 233),      // snow storm
            success: Color::Rgb(163, 190, 140),   // green
            error: Color::Rgb(191, 97, 106),      // red
            search: Color::Rgb(235, 203, 139),    // yellow
            bar_bg: Color::Rgb(59, 66, 82),       // nord1
            mode_fg: Color::Rgb(46, 52, 64),      // nord0
        }
    }

    fn solarized() -> Self {
        Self {
            accent: Color::Rgb(38, 139, 210),     // blue
            dimmed: Color::Rgb(88, 110, 117),     // base01
            text: Color::Rgb(131, 148, 150),      // base0
            success: Color::Rgb(133, 153, 0),     // green
            error: Color::Rgb(220, 50, 47),       // red
            search: Color::Rgb(181, 137, 0),      // yellow
            bar_bg: Color::Rgb(7, 54, 66),        // base02
            mode_fg: Color::Rgb(0, 43, 54),       // base03
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            accent: Color::Cyan,
            dimmed: Color::DarkGray,
            text: Color::White,
            success: Color::Green,
            error: Color::Red,
            search: Color::Yellow,
            bar_bg: Color::DarkGray,
            mode_fg: Color::Black,
        }
    }
}
