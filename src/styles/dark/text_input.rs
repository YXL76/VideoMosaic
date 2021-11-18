use {
    crate::styles::{colors, spacings},
    iced::{text_input, Color},
};

pub struct TextInput;

impl text_input::StyleSheet for TextInput {
    fn active(&self) -> text_input::Style {
        text_input::Style {
            background: colors::blue_gray::_700.into(),
            border_radius: spacings::_2 as f32,
            border_width: 0.,
            border_color: Color::TRANSPARENT,
        }
    }

    fn focused(&self) -> text_input::Style {
        text_input::Style {
            border_width: 2.,
            border_color: colors::blue::_500.into(),
            ..self.active()
        }
    }

    fn placeholder_color(&self) -> Color {
        colors::blue_gray::_500
    }

    fn value_color(&self) -> Color {
        colors::gray::_100
    }

    fn selection_color(&self) -> Color {
        colors::blue::_500
    }

    fn hovered(&self) -> text_input::Style {
        text_input::Style {
            border_width: 2.,
            border_color: Color {
                a: 0.7,
                ..colors::blue::_500.into()
            },
            ..self.focused()
        }
    }
}
