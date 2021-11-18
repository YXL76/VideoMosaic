mod states;
mod steps;
mod streams;
mod styles;
mod widgets;

use {
    iced::{
        button, executor, window, Application, Column, Command, Container, Element, Length, Row,
        Settings, Space, Subscription, Text,
    },
    iced_native::subscription,
    image,
    image_diff::ProcessWrapper,
    rfd::{AsyncMessageDialog, FileDialog, MessageButtons, MessageDialog, MessageLevel},
    states::{State, EN, IMAGE_FILTER, VIDEO_FILTER, ZH_CN},
    std::{
        borrow::Cow,
        ffi::OsStr,
        fs::{create_dir, read_dir, remove_dir},
        path::{Path, PathBuf},
    },
    steps::{StepMessage, Steps, TargetType},
    streams::{crawler, process},
    styles::{spacings, Theme},
    widgets::{btn_icon, btn_text, pri_btn, rou_btn, sec_btn},
};

pub fn main() -> iced::Result {
    image_diff::init().unwrap();
    MosaicVideo::run(Settings {
        window: window::Settings {
            position: window::Position::Centered,
            decorations: false,
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
    exit_btn: button::State,
    back_btn: button::State,
    next_btn: button::State,

    should_exit: bool,

    steps: Steps<'a>,
}

#[derive(Debug, Clone)]
enum Message {
    I18nPressed,
    ThemePressed,
    ExitPressed,
    BackPressed,
    NextPressed,
    NativeEvent(iced_native::Event),
    Step(StepMessage),
}

impl<'a> Application for MosaicVideo<'a> {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        (Self::default(), Command::none())
    }

    fn title(&self) -> String {
        format!("{} - Mosaic Video", self.steps.title(&self.state))
    }

    fn update(&mut self, message: Message) -> Command<Self::Message> {
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
            Message::ExitPressed => self.try_exit(),
            Message::BackPressed => steps.back(state),
            Message::NextPressed => steps.next(state),

            Message::NativeEvent(ev) => {
                if let iced_native::Event::Window(iced_native::window::Event::CloseRequested) = ev {
                    self.try_exit();
                }
            }

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

                StepMessage::AddLocalLibrary => {
                    if !state.is_full() {
                        if let Some(path) = FileDialog::new().pick_folder().as_ref() {
                            self.add_library(path);
                        }
                    }
                }
                StepMessage::DeleteLocalLibrary(folder) => {
                    if MessageDialog::new()
                        .set_level(MessageLevel::Warning)
                        .set_title(state.i18n.delete)
                        .set_description(state.i18n.delete_desc)
                        .set_buttons(MessageButtons::YesNo)
                        .show()
                    {
                        state.libraries.remove(&folder);
                    }
                }

                StepMessage::AddCrawler => {
                    if !state.is_full() {
                        state.pending.push(String::new())
                    }
                }
                StepMessage::EditCrawler(idx, text) => state.pending[idx] = text,
                StepMessage::StartCrawler(idx) => {
                    if !state.pending[idx].is_empty() {
                        let keyword = state.pending.remove(idx);
                        let mut i = 0;
                        let folder = loop {
                            i += 1;
                            let folder = PathBuf::from(format!("{}-{}", keyword, i));
                            if !folder.exists() {
                                break folder;
                            }
                        };
                        create_dir(&folder).unwrap();
                        state.crawler_id += 1;
                        state.crawlers.insert(
                            state.crawler_id,
                            crawler::Crawler::new(state.crawler_id, keyword, 100, folder),
                        );
                    }
                }
                StepMessage::DeleteCrawler(idx) => {
                    state.pending.remove(idx);
                }
                StepMessage::CrawlerMessage(ev) => match ev {
                    crawler::Progress::Downloading(id) => {
                        if let Some(v) = state.crawlers.get_mut(&id) {
                            v.add();
                        }
                    }
                    crawler::Progress::Finished(id) => {
                        if let Some(v) = state.crawlers.remove(&id) {
                            self.add_library(v.folder());
                        }
                    }
                    crawler::Progress::Error(id, err) => {
                        error_dialog(state.i18n.error, err.into());
                        if let Some(v) = state.crawlers.remove(&id) {
                            let _ = remove_dir(v.folder());
                        }
                    }
                    crawler::Progress::None => (),
                },

                StepMessage::CalculationUnit(item) => state.calc_unit = item,
                StepMessage::ColorSpace(item) => state.color_space = item,
                StepMessage::DistanceAlgorithm(item) => state.dist_algo = item,

                StepMessage::Start => {
                    if let Ok(img) = image::open(&state.target_path) {
                        let img = img.into_rgb8();
                        let len = state.libraries.values().fold(0, |s, i| s + i.len());
                        let library =
                            state
                                .libraries
                                .values()
                                .fold(Vec::with_capacity(len), |mut vec, i| {
                                    vec.extend_from_slice(i);
                                    vec
                                });
                        let masks = ProcessWrapper::mask(50, &img);
                        for i in state.percentage.iter_mut() {
                            *i = 0.;
                        }
                        state.step[0] = 100. / library.len() as f32;
                        state.step[1] = 100. / masks.len() as f32;
                        state.process = Some(process::Process::new(
                            50,
                            state.calc_unit,
                            state.color_space,
                            state.dist_algo,
                            img,
                            library,
                            masks,
                        ))
                    }
                }

                StepMessage::ProcessMessage(ev) => match ev {
                    process::Progress::Indexing => state.percentage[0] += state.step[0],
                    process::Progress::Filling => {
                        state.percentage[0] = 100.;
                        state.percentage[1] += state.step[1]
                    }
                    process::Progress::Finished(img_buf) => {
                        state.percentage[1] = 100.;
                        state.process = None;
                        let ext = OsStr::new("png");
                        let mut path = state.target_path.clone();
                        let mut base = state.target_path.file_stem().unwrap().to_os_string();
                        base.push("-mosaic");
                        path.set_file_name(&base);
                        path.set_extension(ext);
                        while path.exists() {
                            base.push("_");
                            path.set_file_name(&base);
                            path.set_extension(ext);
                        }
                        match img_buf.save(&path) {
                            Ok(_) => info_dialog(state.i18n.info, path.to_string_lossy()),
                            Err(e) => error_dialog(state.i18n.error, e.to_string().into()),
                        };
                    }
                    process::Progress::Error(err) => {
                        error_dialog(state.i18n.error, err.into());
                        state.process = None;
                    }
                    process::Progress::None => (),
                },
            },
        }

        Command::none()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        Subscription::batch([
            subscription::events().map(Message::NativeEvent),
            self.steps.subscription(&self.state).map(Message::Step),
        ])
    }

    fn view(&mut self) -> Element<Message> {
        let title = self.title();
        let Self {
            state,
            i18n_btn,
            theme_btn,
            exit_btn,
            back_btn,
            next_btn,
            steps,
            ..
        } = self;

        let i18n_l = btn_text(state.i18n.symbol);
        let theme_l = btn_icon(state.theme.symbol());
        let close_l = btn_icon("\u{f156}");
        let header = Row::new()
            .spacing(spacings::_3)
            .push(Text::new(title).size(spacings::_12).width(Length::Fill))
            .push(
                rou_btn(i18n_btn, i18n_l, spacings::_12, &state.theme)
                    .on_press(Message::I18nPressed),
            )
            .push(
                rou_btn(theme_btn, theme_l, spacings::_12, &state.theme)
                    .on_press(Message::ThemePressed),
            )
            .push(
                rou_btn(exit_btn, close_l, spacings::_12, &state.theme)
                    .on_press(Message::ExitPressed),
            );

        let back_i = btn_icon("\u{f141} ");
        let back_l = btn_text(state.i18n.back);
        let next_i = btn_icon(" \u{f142}");
        let next_l = btn_text(state.i18n.next);
        let btn_w = spacings::_24;
        let control_items: Option<[Element<_>; 3]> =
            match (steps.can_back(state), steps.can_next(state)) {
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

    fn should_exit(&self) -> bool {
        self.should_exit
    }
}

impl MosaicVideo<'_> {
    fn add_library(&mut self, path: &Path) {
        let Self { state, .. } = self;
        let entries = match read_dir(path) {
            Ok(entries) => entries,
            Err(err) => {
                error_dialog(state.i18n.error, err.to_string().into());
                return;
            }
        };
        let entries = entries
            .filter_map(|res| match res.as_ref() {
                Ok(entry) => {
                    let path = entry.path();
                    let ext = path
                        .extension()
                        .unwrap_or_default()
                        .to_str()
                        .unwrap_or_default();
                    if path.is_file() && IMAGE_FILTER.contains(&ext) {
                        Some(path)
                    } else {
                        None
                    }
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        if !entries.is_empty() {
            state.libraries.insert(path.to_path_buf(), entries);
        }
    }

    fn try_exit(&mut self) {
        if MessageDialog::new()
            .set_level(MessageLevel::Warning)
            .set_title(self.state.i18n.exit)
            .set_description(self.state.i18n.exit_hint)
            .set_buttons(MessageButtons::YesNo)
            .show()
        {
            self.should_exit = true;
        }
    }
}

fn info_dialog(title: &'static str, text: Cow<'_, str>) {
    async_dialog(title, MessageLevel::Info, text);
}

fn error_dialog(title: &'static str, text: Cow<'_, str>) {
    async_dialog(title, MessageLevel::Error, text);
}

fn async_dialog(title: &'static str, level: MessageLevel, text: Cow<'_, str>) {
    let _ = AsyncMessageDialog::new()
        .set_level(level)
        .set_title(title)
        .set_description(&text.into_owned())
        .show();
}
