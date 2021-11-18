use iced::Subscription;
use {
    super::{Step, StepMessage},
    crate::{
        states::State,
        styles::spacings,
        widgets::{btn_icon, btn_text, pri_btn},
    },
    iced::{
        button, scrollable, Alignment, Column, Element, Length, ProgressBar, Row, Rule, Scrollable,
    },
};

#[derive(Default)]
pub struct ProcessPreview {
    toggle: button::State,
    scroll: scrollable::State,
}

impl<'a> Step<'a> for ProcessPreview {
    fn title(&self, state: &State) -> &str {
        state.i18n.process
    }

    fn can_back(&self, state: &State) -> bool {
        state.process.is_none()
    }

    fn can_next(&self, state: &State) -> bool {
        state.process.is_none()
    }

    fn subscription(&self, state: &State) -> Subscription<StepMessage> {
        match state.process.as_ref() {
            Some(proc) => proc.subscription().map(StepMessage::ProcessMessage),
            None => Subscription::none(),
        }
    }

    fn view(&mut self, state: &State) -> Element<StepMessage> {
        let Self { toggle, scroll } = self;

        let mut btn = pri_btn(
            toggle,
            btn_icon("\u{f40a} "),
            btn_text(state.i18n.start),
            spacings::_32,
            &state.theme,
        );
        if state.process.is_some() {
            btn = btn.on_press(StepMessage::Start);
        }

        let progresses = state.percentage.iter().fold(
            Column::new().spacing(spacings::_6).width(Length::Fill),
            |col, &perc| col.push(ProgressBar::new(0.0..=100.0, perc).style(state.theme)),
        );

        Scrollable::new(scroll)
            .push(
                Row::new()
                    .spacing(spacings::_8)
                    .align_items(Alignment::Center)
                    .push(btn)
                    .push(progresses),
            )
            .push(Rule::horizontal(spacings::_4).style(state.theme))
            .height(Length::Fill)
            .style(state.theme)
            .into()
    }
}
