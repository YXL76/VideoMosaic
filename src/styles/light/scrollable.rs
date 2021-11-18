use {
    crate::styles::{colors, spacings},
    iced::{scrollable, Color},
};

pub struct Scrollable;

impl scrollable::StyleSheet for Scrollable {
    fn active(&self) -> scrollable::Scrollbar {
        scrollable::Scrollbar {
            background: colors::cool_gray::_100.into(),
            border_radius: spacings::_1 as f32,
            border_width: 0.,
            border_color: Color::TRANSPARENT,
            scroller: scrollable::Scroller {
                color: colors::sky::_500,
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
                ..colors::cool_gray::_100
            }
            .into(),
            scroller: scrollable::Scroller {
                color: colors::sky::_700,
                ..active.scroller
            },
            ..active
        }
    }

    fn dragging(&self) -> scrollable::Scrollbar {
        let hovered = self.hovered();

        scrollable::Scrollbar {
            scroller: scrollable::Scroller {
                color: colors::sky::_300,
                ..hovered.scroller
            },
            ..hovered
        }
    }
}
