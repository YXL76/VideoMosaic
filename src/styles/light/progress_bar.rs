use {
    crate::styles::{colors, spacings},
    iced::progress_bar,
};

pub struct ProgressBar;

impl progress_bar::StyleSheet for ProgressBar {
    fn style(&self) -> progress_bar::Style {
        progress_bar::Style {
            background: colors::cool_gray::_100.into(),
            bar: colors::sky::_500.into(),
            border_radius: spacings::_3 as f32,
        }
    }
}
