use {
    super::{Step, StepMessage},
    crate::{
        states::State,
        styles::spacings,
        widgets::{btn_icon, btn_text, pri_btn, tra_btn},
    },
    iced::{button, scrollable, Column, Element, Length, Row, Scrollable, Text},
};

#[derive(Default)]
pub struct ChooseLibrary {
    left_scroll: scrollable::State,
    right_scroll: scrollable::State,
    local_btn: button::State,
    library_btn: Vec<button::State>,
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
            library_btn,
        } = self;

        while library_btn.len() > state.libraries.len() {
            library_btn.pop();
        }
        while library_btn.len() < state.libraries.len() {
            library_btn.push(button::State::default());
        }

        let left_side = state.libraries.keys().zip(library_btn.iter_mut()).fold(
            Scrollable::new(left_scroll).spacing(spacings::_2),
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
        let right_side =
            state
                .libraries
                .values()
                .fold(Scrollable::new(right_scroll), |scroll, files| {
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
                });

        let cl_i = btn_icon("\u{f254} ");
        let cl_l = btn_text(state.i18n.choose_library);
        let left_side = Column::new()
            .spacing(spacings::_4)
            .push(
                pri_btn(local_btn, cl_i, cl_l, spacings::_32, &state.theme)
                    .on_press(StepMessage::AddLocalLibrary),
            )
            .push(left_side);

        Row::new()
            .spacing(spacings::_4)
            .push(left_side.width(Length::FillPortion(7)))
            .push(right_side.width(Length::FillPortion(10)))
            .height(Length::Fill)
            .into()
    }
}
