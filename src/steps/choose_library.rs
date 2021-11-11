use {
    super::{Step, StepMessage},
    crate::{
        states::{State, LIBRARY_BTN_CNT},
        styles::spacings,
        widgets::{btn_icon, btn_text, pri_btn, tra_btn},
    },
    iced::{button, scrollable, Column, Container, Element, Length, Row, Scrollable, Text},
};

#[derive(Default)]
pub struct ChooseLibrary {
    left_scroll: scrollable::State,
    right_scroll: scrollable::State,
    local_btn: button::State,
    spider_btn: button::State,
    library_btn: [button::State; LIBRARY_BTN_CNT],
}

impl<'a> Step<'a> for ChooseLibrary {
    fn title(&self, state: &State) -> &str {
        state.i18n.choose_library
    }

    fn can_next(&self, state: &State) -> bool {
        state.libraries.len() > 0
    }

    fn view(&mut self, state: &State) -> Element<StepMessage> {
        let Self {
            left_scroll,
            right_scroll,
            local_btn,
            spider_btn,
            library_btn,
        } = self;

        let left_side = state.libraries.keys().zip(library_btn.iter_mut()).fold(
            Scrollable::new(left_scroll).spacing(spacings::_4),
            |scroll, (library, btn)| {
                let icon = btn_icon("\u{f76f} ");
                let label = btn_text(library.to_str().unwrap_or_default());
                scroll.push(
                    tra_btn(btn, icon, label, spacings::_128, &state.theme)
                        .on_press(StepMessage::DeleteLocalLibrary(library.into())),
                )
            },
        );

        let mut count = 0;
        let right_side = state.libraries.values().fold(
            Scrollable::new(right_scroll)
                .width(Length::Fill)
                .padding(spacings::_6),
            |scroll, files| {
                files.iter().fold(scroll, |scroll, file| {
                    count += 1;
                    let row = Row::new()
                        .push(
                            Text::new(format!("{:0>3}.", count))
                                .width(Length::Units(spacings::_10))
                                .size(spacings::_6),
                        )
                        .push(Text::new(file.to_str().unwrap_or_default()).size(spacings::_6));
                    scroll.push(row)
                })
            },
        );

        let cl_l = btn_text(state.i18n.choose_library);
        let dp_l = btn_text(state.i18n.from_the_web);
        let left_ctl = Row::new()
            .spacing(spacings::_8)
            .push(
                pri_btn(local_btn, btn_icon("\u{f254} "), cl_l, 0, &state.theme)
                    .width(Length::FillPortion(1))
                    .on_press(StepMessage::AddLocalLibrary),
            )
            .push(
                pri_btn(spider_btn, btn_icon("\u{f0e4} "), dp_l, 0, &state.theme)
                    .width(Length::FillPortion(1))
                    .on_press(StepMessage::Spider),
            );
        let left_side = Column::new()
            .spacing(spacings::_8)
            .push(left_ctl)
            .push(left_side);

        Row::new()
            .spacing(spacings::_8)
            .push(left_side.width(Length::FillPortion(7)))
            .push(
                Container::new(right_side)
                    .width(Length::FillPortion(10))
                    .height(Length::Fill)
                    .style(state.theme.inner_cont()),
            )
            .height(Length::Fill)
            .into()
    }
}
