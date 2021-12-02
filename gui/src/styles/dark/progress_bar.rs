use {
    crate::styles::{colors, spacings},
    iced::progress_bar,
};

pub struct ProgressBar;

impl progress_bar::StyleSheet for ProgressBar {
    fn style(&self) -> progress_bar::Style {
        progress_bar::Style {
            background: colors::blue_gray::_700.into(),
            bar: colors::blue::_500.into(),
            border_radius: spacings::_3 as f32,
        }
    }
}
