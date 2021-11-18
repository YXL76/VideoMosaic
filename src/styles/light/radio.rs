use {
    crate::styles::colors,
    iced::{radio, Color},
};

pub struct Radio;

impl radio::StyleSheet for Radio {
    fn active(&self) -> radio::Style {
        radio::Style {
            background: Color::WHITE.into(),
            dot_color: colors::sky::_500.into(),
            border_width: 1.,
            border_color: colors::sky::_500.into(),
        }
    }

    fn hovered(&self) -> radio::Style {
        radio::Style {
            background: colors::cool_gray::_100.into(),
            ..self.active()
        }
    }
}
