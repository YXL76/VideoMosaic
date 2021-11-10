use {super::super::colors, iced::container};

pub struct Inner;

impl container::StyleSheet for Inner {
    fn style(&self) -> container::Style {
        container::Style {
            background: Some(colors::cool_gray::_100.into()),
            ..container::Style::default()
        }
    }
}
