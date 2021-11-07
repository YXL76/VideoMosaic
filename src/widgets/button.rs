use {
    crate::styles::{spacings, Theme},
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
    style: Box<dyn button::StyleSheet>,
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
    theme: &Theme,
) -> Button<'a, Message> {
    btn(state, label, len, theme.primary_btn())
}

pub fn sec_btn<'a, Message: Clone>(
    state: &'a mut button::State,
    label: &str,
    len: u16,
    theme: &Theme,
) -> Button<'a, Message> {
    btn(state, label, len, theme.secondary_btn())
}

pub fn rou_btn<'a, Message: Clone>(
    state: &'a mut button::State,
    label: &str,
    len: u16,
    theme: &Theme,
) -> Button<'a, Message> {
    Button::new(state, center_text(label))
        .width(Length::Units(len))
        .height(Length::Units(len))
        .style(theme.transparency_btn())
}
