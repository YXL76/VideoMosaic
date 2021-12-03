use {
    crate::styles::colors,
    iced::{toggler, Color},
};

pub struct Toggler;

impl toggler::StyleSheet for Toggler {
    fn active(&self, is_active: bool) -> toggler::Style {
        toggler::Style {
            background: match is_active {
                true => colors::sky::_500,
                false => colors::cool_gray::_100,
            },
            background_border: None,
            foreground: match is_active {
                true => Color::WHITE,
                false => colors::sky::_500,
            },
            foreground_border: None,
        }
    }

    fn hovered(&self, is_active: bool) -> toggler::Style {
        toggler::Style {
            foreground: Color {
                a: 0.6,
                ..match is_active {
                    true => Color::WHITE,
                    false => colors::sky::_500,
                }
            },
            ..self.active(is_active)
        }
    }
}
