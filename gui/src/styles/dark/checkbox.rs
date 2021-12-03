use {
    crate::styles::colors,
    iced::{checkbox, Color},
};

pub struct Checkbox;

impl checkbox::StyleSheet for Checkbox {
    fn active(&self, is_checked: bool) -> checkbox::Style {
        checkbox::Style {
            background: match is_checked {
                true => colors::blue::_500,
                false => colors::blue_gray::_700,
            }
            .into(),
            checkmark_color: Color::WHITE,
            text_color: colors::gray::_100,
            border_radius: 2.,
            border_width: 1.,
            border_color: colors::blue::_500,
        }
    }

    fn hovered(&self, is_checked: bool) -> checkbox::Style {
        checkbox::Style {
            background: Color {
                a: 0.8,
                ..match is_checked {
                    true => colors::blue::_500,
                    false => colors::blue_gray::_700,
                }
            }
            .into(),
            ..self.active(is_checked)
        }
    }
}
