use {
    super::super::colors,
    iced::{container, Color},
};

pub struct Container;

impl container::StyleSheet for Container {
    fn style(&self) -> container::Style {
        container::Style {
            background: Some(colors::blue_gray::_800.into()),
            text_color: Some(Color::WHITE),
            ..container::Style::default()
        }
    }
}
