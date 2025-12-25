use iced::{Color, Theme, Background, Border};
use iced::widget::{container, text, Text};

/// THE PALETTE
/// Sci-Fi / Cyberpunk Color Scheme
pub struct Palette;

impl Palette {
    // Base Tones
    pub const BACKGROUND: Color = Color::from_rgb(0.05, 0.07, 0.09); // #0D1117
    pub const SURFACE: Color = Color::from_rgb(0.09, 0.11, 0.13);    // #161B22
    pub const TEXT_MAIN: Color = Color::from_rgb(0.9, 0.9, 0.9);
    pub const TEXT_DIM: Color = Color::from_rgb(0.55, 0.58, 0.62);   // #8B949E

    // Status Indicators
    pub const RED: Color = Color::from_rgb(1.0, 0.48, 0.45);     // Danger
    pub const GREEN: Color = Color::from_rgb(0.22, 0.83, 0.33);  // Good
    pub const BLUE: Color = Color::from_rgb(0.35, 0.65, 1.0);    // Info
    pub const PURPLE: Color = Color::from_rgb(0.82, 0.66, 1.0);  // AI
    pub const YELLOW: Color = Color::from_rgb(0.86, 0.67, 0.04); // Warning
    pub const ORANGE: Color = Color::from_rgb(0.82, 0.60, 0.40); // Wait
}

/// STYLE BUILDERS

// The Main Window Background
pub fn style_background(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Palette::BACKGROUND)),
        ..Default::default()
    }
}

// The "Status Card" (Bordered Box)
pub fn style_glass_card(accent: Color) -> impl Fn(&Theme) -> container::Style {
    move |_theme| container::Style {
        background: Some(Background::Color(Palette::SURFACE)),
        border: Border {
            color: accent,
            width: 2.0,
            radius: 12.0.into(),
        },
        ..Default::default()
    }
}

/// WIDGET HELPERS

pub fn label_header(content: &str) -> Text {
    text(content).size(14).color(Palette::TEXT_DIM)
}

pub fn label_main(content: &str, color: Color) -> Text {
    text(content).size(24).color(color)
}