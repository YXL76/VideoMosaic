use {
    crate::styles::{colors, spacings},
    iced::{text_input, Color},
};

pub struct TextInput;

impl text_input::StyleSheet for TextInput {
    fn active(&self) -> text_input::Style {
        text_input::Style {
            background: colors::cool_gray::_100.into(),
            border_radius: spacings::_2 as f32,
            border_width: 0.,
            border_color: Color::TRANSPARENT,
        }
    }

    fn focused(&self) -> text_input::Style {
        text_input::Style {
            border_width: 2.,
            border_color: colors::sky::_500,
            ..self.active()
        }
    }

    fn placeholder_color(&self) -> Color {
        colors::cool_gray::_300
    }

    fn value_color(&self) -> Color {
        colors::cool_gray::_800
    }

    fn selection_color(&self) -> Color {
        colors::sky::_500
    }

    fn hovered(&self) -> text_input::Style {
        text_input::Style {
            border_width: 2.,
            border_color: Color {
                a: 0.7,
                ..colors::sky::_500
            },
            ..self.focused()
        }
    }
}
