use {
    super::super::{colors, spacings},
    iced::{button, Background, Color, Vector},
};

pub enum Button {
    Primary,
    Secondary,
    Transparency,
}

impl button::StyleSheet for Button {
    fn active(&self) -> button::Style {
        button::Style {
            background: Some(Background::Color(match self {
                Button::Primary => colors::blue::_500,
                Button::Secondary => colors::cool_gray::_500,
                Button::Transparency => Color::WHITE,
            })),
            border_radius: spacings::_3 as f32,
            shadow_offset: Vector::new(1.0, 1.0),
            text_color: match self {
                Button::Transparency => colors::cool_gray::_800,
                _ => colors::gray::_100,
            },
            ..button::Style::default()
        }
    }

    fn hovered(&self) -> button::Style {
        let active = self.active();
        button::Style {
            text_color: match self {
                Button::Transparency => colors::cool_gray::_800,
                _ => Color::WHITE,
            },
            shadow_offset: Vector::new(1.0, 2.0),
            ..active
        }
    }
}
