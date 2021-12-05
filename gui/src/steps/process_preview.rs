use iced::Subscription;
use {
    super::{Step, StepMessage},
    crate::{states::State, styles::spacings, widgets::pri_btn},
    iced::{
        button, scrollable, Alignment, Column, Element, Image, Length, ProgressBar, Row, Rule,
        Scrollable, Text,
    },
};

#[derive(Default, Copy, Clone)]
pub struct ProcessPreview {
    toggle: button::State,
    scroll: scrollable::State,
}

impl<'a> Step<'a> for ProcessPreview {
    fn title(&self, state: &State) -> &str {
        state.i18n.process_preview
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
            format!("\u{f909} {}", state.i18n.start),
            spacings::_32,
            &state.theme,
        );
        if state.process.is_none() {
            btn = btn.on_press(StepMessage::Start);
        }

        let labels = [state.i18n.index, state.i18n.fill, state.i18n.composite]
            .into_iter()
            .fold(Column::new().spacing(spacings::_8), |col, label| {
                col.push(Text::new(label).size(spacings::_6))
            });

        let progresses = state.percentage.iter().fold(
            Column::new().spacing(spacings::_6).width(Length::Fill),
            |col, &perc| {
                col.push(
                    ProgressBar::new(0.0..=100.0, perc)
                        .height(Length::Units(spacings::_8))
                        .style(state.theme),
                )
            },
        );

        let mut container = Scrollable::new(scroll)
            .padding(spacings::_6)
            .spacing(spacings::_6)
            .push(
                Row::new()
                    .spacing(spacings::_8)
                    .align_items(Alignment::Center)
                    .push(btn)
                    .push(labels)
                    .push(progresses),
            )
            .push(Rule::horizontal(spacings::_16).style(state.theme))
            .height(Length::Fill)
            .align_items(Alignment::Center)
            .style(state.theme);

        if let Some(img) = state.result_preview.as_ref() {
            container = container
                .push(Image::new(img.clone()).width(Length::Fill))
                .push(Text::new(state.result_path.to_str().unwrap_or("")));
        }

        container.into()
    }
}
