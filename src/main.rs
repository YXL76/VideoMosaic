#![feature(once_cell)]

mod states;
mod steps;
mod styles;
mod widgets;

use {
    iced::{
        button, scrollable, window, Column, Container, Element, Length, Row, Sandbox, Scrollable,
        Settings, Space,
    },
    states::{TargetType, STATE},
    steps::{StepMessage, Steps},
    styles::{fonts, spacings},
    widgets::{pri_btn, sec_btn},
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
    scroll: scrollable::State,
    back_btn: button::State,
    next_btn: button::State,

    steps: Steps<'a>,
}

#[derive(Debug, Clone, Copy)]
enum Message {
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
        format!("{} - Mosaic Video", self.steps.title())
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::BackPressed => {
                self.steps.back();
            }
            Message::NextPressed => {
                self.steps.next();
            }
            Message::StepMessage(step_message) => match step_message {
                StepMessage::TargetType(target_type) => match target_type {
                    TargetType::Image => {
                        let res = rfd::FileDialog::new()
                            .add_filter("image", &["png", "jpg", "jpeg"])
                            .set_title("Choose Image")
                            .pick_file();
                        let mut guard = STATE.write().unwrap();
                        if let Some(path) = res {
                            guard.target_path = path;
                            guard.target_type = target_type;
                        } else {
                            guard.target_type = TargetType::None;
                        }
                    }
                    TargetType::Video => {
                        let res = rfd::FileDialog::new()
                            .add_filter("video", &["mp4"])
                            .set_title("Choose Video")
                            .pick_file();
                        let mut guard = STATE.write().unwrap();
                        if let Some(path) = res {
                            guard.target_path = path;
                            guard.target_type = target_type;
                        } else {
                            guard.target_type = TargetType::None;
                        }
                    }
                    TargetType::None => {
                        STATE.write().unwrap().target_type = target_type;
                    }
                },
            },
        }
    }

    fn view(&mut self) -> Element<Message> {
        const BACK_LABEL: &str = " Back";
        const NEXT_LABEL: &str = "Next ";

        let controls = match (self.steps.can_back(), self.steps.can_next()) {
            (true, true) => Row::new()
                .push(
                    sec_btn(&mut self.back_btn, BACK_LABEL, spacings::_24)
                        .on_press(Message::BackPressed),
                )
                .push(Space::with_width(Length::Units(10)))
                .push(
                    pri_btn(&mut self.next_btn, NEXT_LABEL, spacings::_24)
                        .on_press(Message::NextPressed),
                ),

            (true, false) => Row::new()
                .push(
                    sec_btn(&mut self.back_btn, BACK_LABEL, spacings::_24)
                        .on_press(Message::BackPressed),
                )
                .push(Space::with_width(Length::Units(10 + spacings::_24))),

            (false, true) => Row::new()
                .push(Space::with_width(Length::Units(10 + spacings::_24)))
                .push(
                    pri_btn(&mut self.next_btn, NEXT_LABEL, spacings::_24)
                        .on_press(Message::NextPressed),
                ),

            (false, false) => Row::new(),
        };

        let scrollable = Scrollable::new(&mut self.scroll)
            .push(Container::new(self.steps.view().map(Message::StepMessage)))
            .height(Length::Fill);

        let content = Column::new()
            .max_width(960)
            .padding(spacings::_16)
            .spacing(spacings::_8)
            .push(scrollable)
            .push(Container::new(controls).width(Length::Fill).center_x());

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .into()
    }
}
