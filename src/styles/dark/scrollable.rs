use {
    crate::styles::{colors, spacings},
    iced::{scrollable, Color},
};

pub struct Scrollable;

impl scrollable::StyleSheet for Scrollable {
    fn active(&self) -> scrollable::Scrollbar {
        scrollable::Scrollbar {
            background: colors::blue_gray::_700.into(),
            border_radius: spacings::_1 as f32,
            border_width: 0.,
            border_color: Color::TRANSPARENT,
            scroller: scrollable::Scroller {
                color: colors::blue::_500,
                border_radius: spacings::_1 as f32,
                border_width: 0.,
                border_color: Color::TRANSPARENT,
            },
        }
    }

    fn hovered(&self) -> scrollable::Scrollbar {
        let active = self.active();

        scrollable::Scrollbar {
            background: Color {
                a: 0.6,
                ..colors::blue_gray::_700
            }
            .into(),
            scroller: scrollable::Scroller {
                color: colors::blue::_700,
                ..active.scroller
            },
            ..active
        }
    }

    fn dragging(&self) -> scrollable::Scrollbar {
        let hovered = self.hovered();

        scrollable::Scrollbar {
            scroller: scrollable::Scroller {
                color: colors::blue::_300,
                ..hovered.scroller
            },
            ..hovered
        }
    }
}
