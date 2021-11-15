mod states;
mod steps;
mod styles;
mod widgets;

use {
    iced::{
        button, window, Column, Container, Element, Length, Row, Sandbox, Settings, Space, Text,
    },
    rfd::{FileDialog, MessageButtons, MessageDialog},
    states::{State, EN, LIBRARY_BTN_CNT, ZH_CN},
    std::fs::read_dir,
    steps::{StepMessage, Steps, TargetType},
    styles::{spacings, Theme},
    widgets::{btn_icon, btn_text, pri_btn, rou_btn, sec_btn},
};

pub fn main() -> iced::Result {
    image_diff::init().unwrap();
    MosaicVideo::run(Settings {
        window: window::Settings {
            position: window::Position::Centered,
            ..window::Settings::default()
        },
        text_multithreading: true,
        antialiasing: false,
        ..Settings::default()
    })
}

#[derive(Default)]
struct MosaicVideo<'a> {
    state: State,

    i18n_btn: button::State,
    theme_btn: button::State,
    back_btn: button::State,
    next_btn: button::State,

    steps: Steps<'a>,
}

#[derive(Debug, Clone)]
enum Message {
    I18nPressed,
    ThemePressed,
    BackPressed,
    NextPressed,
    Step(StepMessage),
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
        const IMAGE_FILTER: [&str; 3] = ["png", "jpg", "jpeg"];
        const VIDEO_FILTER: [&str; 1] = ["mp4"];

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
            Message::NextPressed => steps.next(state),

            Message::Step(step_message) => match step_message {
                StepMessage::TargetType(target_type) => {
                    let pick_res = match target_type {
                        TargetType::Image => FileDialog::new()
                            .add_filter("image", &IMAGE_FILTER)
                            .set_title("Choose Image")
                            .pick_file(),
                        TargetType::Video => FileDialog::new()
                            .add_filter("video", &VIDEO_FILTER)
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

                StepMessage::AddLocalLibrary if state.libraries.len() < LIBRARY_BTN_CNT => {
                    if let Some(folder) = FileDialog::new().pick_folder().as_ref() {
                        let dir = folder.clone();
                        if let Ok(entries) = read_dir(folder) {
                            let entries = entries
                                .filter_map(|res| {
                                    if let Ok(entry) = res.as_ref() {
                                        let path = entry.path();
                                        let ext = path
                                            .extension()
                                            .unwrap_or_default()
                                            .to_str()
                                            .unwrap_or_default();
                                        if path.is_file() && IMAGE_FILTER.contains(&ext) {
                                            return Some(path);
                                        }
                                    };
                                    None
                                })
                                .collect::<Vec<_>>();
                            if !entries.is_empty() {
                                state.libraries.insert(dir, entries);
                            }
                        }
                    }
                }

                StepMessage::DeleteLocalLibrary(folder) => {
                    if MessageDialog::new()
                        .set_title(state.i18n.delete)
                        .set_description(state.i18n.delete_desc)
                        .set_buttons(MessageButtons::YesNo)
                        .show()
                    {
                        state.libraries.remove(&folder);
                    }
                }

                StepMessage::Spider => (),

                StepMessage::CalculationUnit(item) => state.calc_unit = item,
                StepMessage::ColorSpace(item) => state.color_space = item,
                StepMessage::DistanceAlgorithm(item) => state.dist_algo = item,

                StepMessage::Start => {
                    let len = state.libraries.values().fold(0, |s, i| s + i.len());
                    let library =
                        state
                            .libraries
                            .values()
                            .fold(Vec::with_capacity(len), |mut vec, i| {
                                vec.extend_from_slice(i);
                                vec
                            });
                    image_diff::ProcessWrapper::new(
                        50,
                        state.calc_unit,
                        state.color_space,
                        state.dist_algo,
                    )
                    .run(&state.target_path, &library)
                    .unwrap()
                    .save("tmp.png")
                    .unwrap();
                }

                _ => (),
            },
        }
    }

    fn view(&mut self) -> Element<Message> {
        let title = self.title();
        let Self {
            state,
            i18n_btn,
            theme_btn,
            back_btn,
            next_btn,
            steps,
        } = self;

        let i18n_l = btn_text(state.i18n.symbol);
        let theme_l = btn_icon(state.theme.symbol());
        let header = Row::new()
            .push(Text::new(title).size(spacings::_12).width(Length::Fill))
            .push(
                rou_btn(i18n_btn, i18n_l, spacings::_12, &state.theme)
                    .on_press(Message::I18nPressed),
            )
            .push(Space::with_width(Length::Units(spacings::_3)))
            .push(
                rou_btn(theme_btn, theme_l, spacings::_12, &state.theme)
                    .on_press(Message::ThemePressed),
            );

        let back_i = btn_icon("\u{f141} ");
        let back_l = btn_text(state.i18n.back);
        let next_i = btn_icon(" \u{f142}");
        let next_l = btn_text(state.i18n.next);
        let btn_w = spacings::_24;
        let control_items: Option<[Element<_>; 3]> = match (steps.can_back(), steps.can_next(state))
        {
            (true, true) => Some([
                sec_btn(back_btn, back_i, back_l, btn_w, &state.theme)
                    .on_press(Message::BackPressed)
                    .into(),
                Space::with_width(Length::Units(10)).into(),
                pri_btn(next_btn, next_l, next_i, btn_w, &state.theme)
                    .on_press(Message::NextPressed)
                    .into(),
            ]),

            (true, false) => Some([
                sec_btn(back_btn, back_i, back_l, btn_w, &state.theme)
                    .on_press(Message::BackPressed)
                    .into(),
                Space::with_width(Length::Units(10)).into(),
                Space::with_width(Length::Units(btn_w)).into(),
            ]),

            (false, true) => Some([
                Space::with_width(Length::Units(10)).into(),
                Space::with_width(Length::Units(btn_w)).into(),
                pri_btn(next_btn, next_l, next_i, btn_w, &state.theme)
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

        let content = Column::new()
            .max_width(1024)
            .padding(spacings::_10)
            .spacing(spacings::_6)
            .push(header)
            .push(steps.view(state).map(Message::Step))
            .push(Container::new(controls).width(Length::Fill).center_x());

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .style(state.theme)
            .into()
    }
}
