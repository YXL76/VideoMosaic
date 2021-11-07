mod states;
mod steps;
mod styles;
mod widgets;

use {
    iced::{
        button, scrollable, window, Column, Container, Element, Length, Row, Sandbox, Scrollable,
        Settings, Space, Text,
    },
    states::{State, TargetType, EN, ZH_CN},
    steps::{StepMessage, Steps},
    styles::{fonts, spacings, Theme},
    widgets::{pri_btn, rou_btn, sec_btn},
};

pub fn main() -> iced::Result {
    MosaicVideo::run(Settings {
        window: window::Settings {
            position: window::Position::Centered,
            ..window::Settings::default()
        },
        text_multithreading: true,
        antialiasing: false,
        default_font: Some(fonts::REGULAR_BYTES),
        ..Settings::default()
    })
}

#[derive(Default)]
struct MosaicVideo<'a> {
    state: State,

    scroll: scrollable::State,
    i18n_btn: button::State,
    theme_btn: button::State,
    back_btn: button::State,
    next_btn: button::State,

    steps: Steps<'a>,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    I18nPressed,
    ThemePressed,
    BackPressed,
    NextPressed,
    StepMessage(StepMessage),
}

impl<'a> Sandbox for MosaicVideo<'a> {
    type Message = Message;

    fn new() -> Self {
        Self::default()
    }

    fn title(&self) -> String {
        format!("{} - Mosaic Video", self.steps.title(&self.state))
    }

    fn update(&mut self, message: Message) {
        let Self { state, steps, .. } = self;

        match message {
            Message::I18nPressed => match state.i18n.symbol {
                "En" => state.i18n = &ZH_CN,
                "中" => state.i18n = &EN,
                _ => (),
            },
            Message::ThemePressed => {
                state.theme = match state.theme {
                    Theme::Light => Theme::Dark,
                    Theme::Dark => Theme::Light,
                }
            }
            Message::BackPressed => steps.back(),
            Message::NextPressed => steps.next(),

            Message::StepMessage(step_message) => match step_message {
                StepMessage::TargetType(target_type) => {
                    let pick_res = match target_type {
                        TargetType::Image => rfd::FileDialog::new()
                            .add_filter("image", &["png", "jpg", "jpeg"])
                            .set_title("Choose Image")
                            .pick_file(),
                        TargetType::Video => rfd::FileDialog::new()
                            .add_filter("video", &["mp4"])
                            .set_title("Choose Video")
                            .pick_file(),
                        TargetType::None => None,
                    };
                    if let Some(path) = pick_res {
                        state.target_path = path;
                        state.target_type = target_type;
                    } else {
                        state.target_type = TargetType::None;
                    }
                }
            },
        }
    }

    fn view(&mut self) -> Element<Message> {
        let title = self.title();
        let Self {
            state,
            scroll,
            i18n_btn,
            theme_btn,
            back_btn,
            next_btn,
            steps,
        } = self;

        let header = Row::new()
            .push(Text::new(title).size(spacings::_12).width(Length::Fill))
            .push(
                rou_btn(i18n_btn, state.i18n.symbol, spacings::_12, &state.theme)
                    .on_press(Message::I18nPressed),
            )
            .push(Space::with_width(Length::Units(spacings::_3)))
            .push(
                rou_btn(theme_btn, state.theme.symbol(), spacings::_12, &state.theme)
                    .on_press(Message::ThemePressed),
            );

        let back_l = state.i18n.back;
        let next_l = state.i18n.next;
        let btn_w = spacings::_24;
        let control_items: Option<[Element<_>; 3]> = match (steps.can_back(), steps.can_next()) {
            (true, true) => Some([
                sec_btn(back_btn, back_l, btn_w, &state.theme)
                    .on_press(Message::BackPressed)
                    .into(),
                Space::with_width(Length::Units(10)).into(),
                pri_btn(next_btn, next_l, btn_w, &state.theme)
                    .on_press(Message::NextPressed)
                    .into(),
            ]),

            (true, false) => Some([
                sec_btn(back_btn, back_l, btn_w, &state.theme)
                    .on_press(Message::BackPressed)
                    .into(),
                Space::with_width(Length::Units(10)).into(),
                Space::with_width(Length::Units(btn_w)).into(),
            ]),

            (false, true) => Some([
                Space::with_width(Length::Units(10)).into(),
                Space::with_width(Length::Units(btn_w)).into(),
                pri_btn(next_btn, next_l, btn_w, &state.theme)
                    .on_press(Message::NextPressed)
                    .into(),
            ]),

            (false, false) => None,
        };
        let mut controls = Row::new();
        if let Some(items) = control_items {
            for item in items {
                controls = controls.push(item);
            }
        }

        let scrollable = Scrollable::new(scroll)
            .push(steps.view(state).map(Message::StepMessage))
            .height(Length::Fill);

        let content = Column::new()
            .max_width(960)
            .padding(spacings::_8)
            .spacing(spacings::_8)
            .push(header)
            .push(scrollable)
            .push(Container::new(controls).width(Length::Fill).center_x());

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .style(state.theme)
            .into()
    }
}
