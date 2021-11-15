use {
    super::{Step, StepMessage},
    crate::{
        states::State,
        styles::spacings,
        widgets::{btn_icon, btn_text, pri_btn},
    },
    iced::{button, scrollable, Element, Length, Scrollable},
};

#[derive(Default)]
pub struct Process {
    toggle: button::State,
    scroll: scrollable::State,
}

impl<'a> Step<'a> for Process {
    fn title(&self, state: &State) -> &str {
        state.i18n.process
    }

    fn can_next(&self, _state: &State) -> bool {
        false
    }

    fn view(&mut self, state: &State) -> Element<StepMessage> {
        let Self { toggle, scroll } = self;

        let btn = pri_btn(
            toggle,
            btn_icon("\u{f40a} "),
            btn_text(state.i18n.start),
            spacings::_32,
            &state.theme,
        )
        .on_press(StepMessage::Start);

        Scrollable::new(scroll)
            .push(btn)
            .height(Length::Fill)
            .into()
    }
}
