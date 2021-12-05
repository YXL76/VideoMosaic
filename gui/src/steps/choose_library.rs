use {
    super::{Step, StepMessage},
    crate::{
        states::{State, LIBRARY_BTN_CNT},
        styles::spacings,
        widgets::{pri_btn, rou_btn, tra_btn},
    },
    iced::{
        button, scrollable, text_input, Column, Container, Element, Length, ProgressBar, Row,
        Scrollable, Subscription, Text, TextInput,
    },
    itertools::izip,
};

#[derive(Default, Clone)]
pub struct ChooseLibrary {
    left_scroll: scrollable::State,
    right_scroll: scrollable::State,
    local_btn: button::State,
    spider_btn: button::State,
    library_btn: [button::State; LIBRARY_BTN_CNT],
    inputs: [(text_input::State, text_input::State); LIBRARY_BTN_CNT],
    delete_btn: [button::State; LIBRARY_BTN_CNT],
}

impl<'a> Step<'a> for ChooseLibrary {
    fn title(&self, state: &State) -> &str {
        state.i18n.choose_library
    }

    fn can_back(&self, state: &State) -> bool {
        state.crawlers.is_empty()
    }

    fn can_next(&self, state: &State) -> bool {
        !state.libraries.is_empty() && state.crawlers.is_empty()
    }

    fn subscription(&self, state: &State) -> Subscription<StepMessage> {
        Subscription::batch(
            state
                .crawlers
                .values()
                .map(|i| i.subscription().map(StepMessage::CrawlerMessage)),
        )
    }

    fn view(&mut self, state: &State) -> Element<StepMessage> {
        let Self {
            left_scroll,
            right_scroll,
            local_btn,
            spider_btn,
            library_btn,
            inputs,
            delete_btn,
        } = self;

        let left_side = izip![
            0..,
            state.pending.iter(),
            inputs.iter_mut(),
            delete_btn.iter_mut()
        ]
        .fold(
            Scrollable::new(left_scroll)
                .spacing(spacings::_4)
                .style(state.theme),
            |scroll, (idx, (keyword, num), (i1, i2), delete)| {
                let row = Row::new()
                    .spacing(spacings::_2)
                    .push(
                        TextInput::new(i1, state.i18n.keyword_prompt, keyword.as_str(), move |s| {
                            StepMessage::EditKeyword(idx, s)
                        })
                        .on_submit(StepMessage::StartCrawler(idx))
                        .width(Length::FillPortion(3))
                        .style(state.theme)
                        .padding(spacings::_1)
                        .size(spacings::_8),
                    )
                    .push(
                        TextInput::new(i2, "", num.as_str(), move |s| {
                            StepMessage::EditNumber(idx, s)
                        })
                        .on_submit(StepMessage::StartCrawler(idx))
                        .width(Length::FillPortion(1))
                        .style(state.theme)
                        .padding(spacings::_1)
                        .size(spacings::_8),
                    )
                    .push(
                        rou_btn(delete, "\u{f6bf}", spacings::_10, state.theme.danger_btn())
                            .on_press(StepMessage::DeleteCrawler(idx)),
                    );
                scroll.push(row)
            },
        );

        let left_side = state.crawlers.values().fold(left_side, |scroll, crawler| {
            scroll.push(
                ProgressBar::new(1.0..=100.0, crawler.percentage())
                    .height(Length::Units(spacings::_10))
                    .style(state.theme),
            )
        });

        let left_side = state.libraries.keys().zip(library_btn.iter_mut()).fold(
            left_side,
            |scroll, (library, btn)| {
                let label = format!("\u{f755} {}", library.to_str().unwrap_or_default());
                scroll.push(
                    tra_btn(btn, label, spacings::_128, &state.theme)
                        .on_press(StepMessage::DeleteLocalLibrary(library.into())),
                )
            },
        );

        let mut count = 0;
        let right_side = state.libraries.values().fold(
            Scrollable::new(right_scroll)
                .width(Length::Fill)
                .padding(spacings::_6)
                .style(state.theme),
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

        let cl_l = format!("\u{f753} {}", state.i18n.choose_library);
        let dp_l = format!("\u{f5e3} {}", state.i18n.from_the_web);
        let left_ctl = Row::new()
            .spacing(spacings::_8)
            .push(
                pri_btn(local_btn, cl_l, 0, &state.theme)
                    .width(Length::FillPortion(1))
                    .on_press(StepMessage::AddLocalLibrary),
            )
            .push(
                pri_btn(spider_btn, dp_l, 0, &state.theme)
                    .width(Length::FillPortion(1))
                    .on_press(StepMessage::AddCrawler),
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
