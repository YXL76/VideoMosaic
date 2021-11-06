use {
    crate::styles::{self, spacings},
    iced::{alignment, button, Button, Length, Text},
};

fn btn<'a, Message: Clone>(
    state: &'a mut button::State,
    label: &str,
    len: u16,
    style: impl button::StyleSheet + 'static,
) -> Button<'a, Message> {
    Button::new(
        state,
        Text::new(label).horizontal_alignment(alignment::Horizontal::Center),
    )
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
