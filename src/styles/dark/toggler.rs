use {
    crate::styles::colors,
    iced::{toggler, Color},
};

pub struct Toggler;

impl toggler::StyleSheet for Toggler {
    fn active(&self, is_active: bool) -> toggler::Style {
        toggler::Style {
            background: match is_active {
                true => colors::blue::_500.into(),
                false => colors::blue_gray::_700.into(),
            },
            background_border: None,
            foreground: match is_active {
                true => colors::blue_gray::_800.into(),
                false => colors::blue::_500.into(),
            },
            foreground_border: None,
        }
    }

    fn hovered(&self, is_active: bool) -> toggler::Style {
        toggler::Style {
            foreground: match is_active {
                true => Color {
                    a: 0.6,
                    ..colors::blue_gray::_800
                },
                false => Color {
                    a: 0.6,
                    ..colors::blue::_500
                },
            }
            .into(),
            ..self.active(is_active)
        }
    }
}