use {
    crate::styles::{spacings, Theme},
    iced::{alignment, button, Button, Length, Text},
};

fn btn<'a, Message: 'a + Clone, T: Into<String>>(
    state: &'a mut button::State,
    label: T,
    len: u16,
    style: Box<dyn button::StyleSheet + 'a>,
) -> Button<'a, Message> {
    Button::new(
        state,
        Text::new(label)
            .vertical_alignment(alignment::Vertical::Center)
            .horizontal_alignment(alignment::Horizontal::Center)
            .width(Length::Fill)
            .height(Length::Fill),
    )
    .padding(spacings::_3)
    .width(Length::Units(len))
    .style(style)
}

#[inline(always)]
pub fn pri_btn<'a, Message: 'a + Clone, T: Into<String>>(
    state: &'a mut button::State,
    label: T,
    len: u16,
    theme: &Theme,
) -> Button<'a, Message> {
    btn(state, label, len, theme.primary_btn())
}

#[inline(always)]
pub fn sec_btn<'a, Message: 'a + Clone, T: Into<String>>(
    state: &'a mut button::State,
    label: T,
    len: u16,
    theme: &Theme,
) -> Button<'a, Message> {
    btn(state, label, len, theme.secondary_btn())
}

#[inline(always)]
pub fn tra_btn<'a, Message: 'a + Clone, T: Into<String>>(
    state: &'a mut button::State,
    label: T,
    len: u16,
    theme: &Theme,
) -> Button<'a, Message> {
    btn(state, label, len, theme.transparency_btn())
}

#[inline(always)]
pub fn rou_btn<'a, Message: 'a + Clone, T: Into<String>>(
    state: &'a mut button::State,
    label: T,
    len: u16,
    style: Box<dyn button::StyleSheet + 'a>,
) -> Button<'a, Message> {
    btn(state, label, len, style).height(Length::Units(len))
}
