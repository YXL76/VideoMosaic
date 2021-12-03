use {
    crate::styles::colors,
    iced::{checkbox, Color},
};

pub struct Checkbox;

impl checkbox::StyleSheet for Checkbox {
    fn active(&self, is_checked: bool) -> checkbox::Style {
        checkbox::Style {
            background: match is_checked {
                true => colors::sky::_500,
                false => colors::cool_gray::_100,
            }
            .into(),
            checkmark_color: Color::WHITE,
            text_color: colors::cool_gray::_800,
            border_radius: 2.,
            border_width: 1.,
            border_color: colors::sky::_500,
        }
    }

    fn hovered(&self, is_checked: bool) -> checkbox::Style {
        checkbox::Style {
            background: Color {
                a: 0.8,
                ..match is_checked {
                    true => colors::sky::_500,
                    false => colors::cool_gray::_100,
                }
            }
            .into(),
            ..self.active(is_checked)
        }
    }
}
