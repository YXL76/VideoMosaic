use {
    crate::styles::{colors, spacings},
    iced::rule,
};

pub struct Rule;

impl rule::StyleSheet for Rule {
    fn style(&self) -> rule::Style {
        rule::Style {
            color: colors::cool_gray::_100,
            width: 2,
            radius: 1.0,
            fill_mode: rule::FillMode::Padded(spacings::_4),
        }
    }
}
