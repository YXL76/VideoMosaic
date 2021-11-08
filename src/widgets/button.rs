use {
    crate::styles::{fonts, spacings, Theme},
    iced::{alignment, button, Button, Container, Length, Row, Text},
};

type BtnText = Text;

pub fn btn_text(label: &str) -> BtnText {
    Text::new(label)
        .horizontal_alignment(alignment::Horizontal::Center)
        .vertical_alignment(alignment::Vertical::Center)
}

pub fn btn_icon(label: &str) -> BtnText {
    Text::new(label)
        .horizontal_alignment(alignment::Horizontal::Center)
        .vertical_alignment(alignment::Vertical::Center)
        .font(fonts::MATERIAL_DESIGN_ICONS)
}

fn btn<'a, Message: 'a + Clone>(
    state: &'a mut button::State,
    left: BtnText,
    right: BtnText,
    len: u16,
    style: Box<dyn button::StyleSheet>,
) -> Button<'a, Message> {
    Button::new(
        state,
        Container::new(
            Row::new()
                .push(left)
                .push(right)
                .align_items(alignment::Alignment::Center),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_y()
        .center_x(),
    )
    .padding(spacings::_3)
    .width(Length::Units(len))
    .style(style)
}

pub fn pri_btn<'a, Message: 'a + Clone>(
    state: &'a mut button::State,
    left: BtnText,
    right: BtnText,
    len: u16,
    theme: &Theme,
) -> Button<'a, Message> {
    btn(state, left, right, len, theme.primary_btn())
}

pub fn sec_btn<'a, Message: 'a + Clone>(
    state: &'a mut button::State,
    left: BtnText,
    right: BtnText,
    len: u16,
    theme: &Theme,
) -> Button<'a, Message> {
    btn(state, left, right, len, theme.secondary_btn())
}

pub fn tra_btn<'a, Message: 'a + Clone>(
    state: &'a mut button::State,
    left: BtnText,
    right: BtnText,
    len: u16,
    theme: &Theme,
) -> Button<'a, Message> {
    btn(state, left, right, len, theme.transparency_btn())
}

pub fn rou_btn<'a, Message: 'a + Clone>(
    state: &'a mut button::State,
    label: BtnText,
    len: u16,
    theme: &Theme,
) -> Button<'a, Message> {
    Button::new(state, label)
        .width(Length::Units(len))
        .height(Length::Units(len))
        .style(theme.transparency_btn())
}
