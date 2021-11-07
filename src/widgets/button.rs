use {
    crate::styles::{self, spacings},
    iced::{alignment, button, Button, Length, Text},
};

fn center_text(label: &str) -> Text {
    Text::new(label)
        .horizontal_alignment(alignment::Horizontal::Center)
        .vertical_alignment(alignment::Vertical::Center)
}

fn btn<'a, Message: Clone>(
    state: &'a mut button::State,
    label: &str,
    len: u16,
    style: impl button::StyleSheet + 'static,
) -> Button<'a, Message> {
    Button::new(state, center_text(label))
        .padding(spacings::_3)
        .width(Length::Units(len))
        .style(style)
}

pub fn pri_btn<'a, Message: Clone>(
    state: &'a mut button::State,
    label: &str,
    len: u16,
) -> Button<'a, Message> {
    btn(state, label, len, styles::Button::Primary)
}

pub fn sec_btn<'a, Message: Clone>(
    state: &'a mut button::State,
    label: &str,
    len: u16,
) -> Button<'a, Message> {
    btn(state, label, len, styles::Button::Secondary)
}

pub fn rou_btn<'a, Message: Clone>(
    state: &'a mut button::State,
    label: &str,
    len: u16,
) -> Button<'a, Message> {
    Button::new(state, center_text(label))
        .width(Length::Units(len))
        .height(Length::Units(len))
        .style(styles::Button::Transparency)
}
