use {
    crate::styles::colors,
    iced::{container, Color},
};

pub struct Outer;

impl container::StyleSheet for Outer {
    fn style(&self) -> container::Style {
        container::Style {
            background: Some(colors::blue_gray::_800.into()),
            text_color: Some(Color::WHITE),
            ..container::Style::default()
        }
    }
}

pub struct Inner;

impl container::StyleSheet for Inner {
    fn style(&self) -> container::Style {
        container::Style {
            background: Some(colors::blue_gray::_700.into()),
            ..Outer.style()
        }
    }
}
