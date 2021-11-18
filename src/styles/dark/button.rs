use {
    crate::styles::{colors, spacings},
    iced::{button, Color, Vector},
};

pub struct Primary;

impl button::StyleSheet for Primary {
    fn active(&self) -> button::Style {
        button::Style {
            background: Some(colors::blue::_500.into()),
            border_radius: spacings::_3 as f32,
            shadow_offset: Vector::new(1.0, 1.0),
            text_color: colors::gray::_100,
            ..button::Style::default()
        }
    }

    fn hovered(&self) -> button::Style {
        button::Style {
            text_color: Color::WHITE,
            shadow_offset: Vector::new(1.0, 2.0),
            ..self.active()
        }
    }
}

pub struct Secondary;

impl button::StyleSheet for Secondary {
    fn active(&self) -> button::Style {
        button::Style {
            background: Some(colors::cool_gray::_600.into()),
            border_radius: spacings::_3 as f32,
            shadow_offset: Vector::new(1.0, 1.0),
            text_color: colors::gray::_100,
            ..button::Style::default()
        }
    }

    fn hovered(&self) -> button::Style {
        button::Style {
            text_color: Color::WHITE,
            shadow_offset: Vector::new(1.0, 2.0),
            ..self.active()
        }
    }
}

pub struct Transparency;

impl button::StyleSheet for Transparency {
    fn active(&self) -> button::Style {
        button::Style {
            background: Some(colors::blue_gray::_800.into()),
            border_radius: spacings::_3 as f32,
            shadow_offset: Vector::new(1.0, 1.0),
            text_color: colors::gray::_100,
            ..button::Style::default()
        }
    }

    fn hovered(&self) -> button::Style {
        button::Style {
            text_color: Color::WHITE,
            shadow_offset: Vector::new(1.0, 2.0),
            ..self.active()
        }
    }
}

pub struct Danger;

impl button::StyleSheet for Danger {
    fn active(&self) -> button::Style {
        button::Style {
            background: Some(colors::red::_500.into()),
            border_radius: spacings::_3 as f32,
            shadow_offset: Vector::new(1.0, 1.0),
            text_color: colors::gray::_100,
            ..button::Style::default()
        }
    }

    fn hovered(&self) -> button::Style {
        button::Style {
            text_color: Color::WHITE,
            shadow_offset: Vector::new(1.0, 2.0),
            ..self.active()
        }
    }
}
