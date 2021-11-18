use {crate::styles::colors, iced::radio};

pub struct Radio;

impl radio::StyleSheet for Radio {
    fn active(&self) -> radio::Style {
        radio::Style {
            background: colors::blue_gray::_800.into(),
            dot_color: colors::blue::_500.into(),
            border_width: 1.,
            border_color: colors::blue::_500.into(),
        }
    }

    fn hovered(&self) -> radio::Style {
        radio::Style {
            background: colors::blue_gray::_700.into(),
            ..self.active()
        }
    }
}
