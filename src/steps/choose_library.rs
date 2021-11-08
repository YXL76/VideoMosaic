use {
    super::{Step, StepMessage},
    crate::{
        states::{State, TargetType},
        styles::spacings,
        widgets::{btn_icon, btn_text, pri_btn},
    },
    iced::{button, scrollable, Column, Element, Length, Row, Scrollable, Text},
};

#[derive(Default)]
pub struct ChooseLibrary {
    left_scroll: scrollable::State,
    right_scroll: scrollable::State,
    local_btn: button::State,
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
        } = self;

        let cl_i = btn_icon("\u{f256} ");
        let cl_l = btn_text(state.i18n.choose_library);

        let mut left_side = Scrollable::new(left_scroll);
        for library in state.libraries.keys() {
            left_side = left_side.push(Text::new(library.to_str().unwrap_or_default()))
        }
        let left_side = Column::new()
            .spacing(spacings::_4)
            .push(
                pri_btn(local_btn, cl_i, cl_l, spacings::_32, &state.theme)
                    .on_press(StepMessage::AddLocalLibrary),
            )
            .push(left_side)
            .width(Length::FillPortion(7));

        let mut right_side = Scrollable::new(right_scroll).width(Length::FillPortion(10));
        for files in state.libraries.values() {
            for file in files {
                right_side = right_side.push(Text::new(file.to_str().unwrap_or_default()))
            }
        }

        Row::new()
            .spacing(spacings::_4)
            .push(left_side)
            .push(right_side)
            .height(Length::Fill)
            .into()
    }
}
