#![windows_subsystem = "windows"]

mod states;
mod steps;
mod streams;
mod styles;
mod widgets;

use {
    iced::{
        button, executor, image::Handle, window, Application, Column, Command, Container, Element,
        Length, Row, Settings, Space, Subscription, Text,
    },
    iced_native::subscription,
    image::{load_from_memory_with_format, ImageFormat},
    mosaic_video_diff::first_frame,
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
    styles::{fonts, spacings, Theme},
    widgets::{pri_btn, rou_btn, sec_btn},
};

pub fn main() -> iced::Result {
    mosaic_video_diff::init().unwrap();
    let icon = load_from_memory_with_format(
        include_bytes!("../../static/images/icon.jpg"),
        ImageFormat::Jpeg,
    )
    .unwrap()
    .into_rgba8()
    .into_raw();
    MosaicVideo::run(Settings {
        window: window::Settings {
            position: window::Position::Centered,
            decorations: false,
            icon: Some(window::Icon::from_rgba(icon, 600, 600).unwrap()),
            ..window::Settings::default()
        },
        default_font: Some(fonts::SARASA_UI_NERD),
        text_multithreading: false,
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
            Message::BackPressed => {
                state.clear();
                steps.back(state);
            }
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
                    state.target_preview = match state.target_type {
                        TargetType::Image | TargetType::Video => {
                            path2handle(&state.target_path, state.target_type == TargetType::Video)
                        }
                        TargetType::None => None,
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
                        state.pending.push((String::new(), String::from("100")))
                    }
                }
                StepMessage::EditKeyword(idx, text) => state.pending[idx].0 = text,
                StepMessage::EditNumber(idx, text) => {
                    if let Ok(num) = text.parse::<usize>() {
                        state.pending[idx].1 = match num {
                            0 => String::from("100"),
                            i if i < 400 => text,
                            _ => String::from("400"),
                        }
                    }
                }
                StepMessage::StartCrawler(idx) => {
                    if !state.pending[idx].0.is_empty() {
                        let (keyword, num) = state.pending.remove(idx);
                        let num = num.parse::<usize>().unwrap();
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
                            crawler::Crawler::new(state.crawler_id, keyword, num, folder),
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

                StepMessage::CalculationUnit(item) => state.config.calc_unit = item,
                StepMessage::ColorSpace(item) => state.config.color_space = item,
                StepMessage::DistanceAlgorithm(item) => state.config.dist_algo = item,
                StepMessage::Filter(item) => state.config.filter = item,
                StepMessage::K(item) => state.config.k = item,
                StepMessage::Hamerly(item) => state.config.hamerly = item,
                StepMessage::Size(item) => state.config.size = item,

                StepMessage::Start => {
                    let video = state.target_type == TargetType::Video;

                    let ext = OsStr::new(if video { "mp4" } else { "png" });
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

                    let len = state.libraries.values().fold(0, |sum, i| sum + i.len());
                    let library =
                        state
                            .libraries
                            .values()
                            .fold(Vec::with_capacity(len), |mut vec, i| {
                                vec.extend_from_slice(i);
                                vec
                            });

                    state.clear();
                    state.result_path = path;
                    state.process = Some(process::Process::new(
                        state.config,
                        state.target_path.to_string_lossy().to_string(),
                        state.result_path.to_string_lossy().to_string(),
                        video,
                        library,
                    ))
                }

                StepMessage::ProcessMessage(ev) => match ev {
                    process::Progress::Started(a, b, c) => {
                        state.process.as_mut().unwrap().clear();
                        state.step[0] = 100. / a;
                        state.step[1] = 100. / b;
                        state.step[2] = 100. / c;
                    }
                    process::Progress::Indexing => state.percentage[0] += state.step[0],
                    process::Progress::Indexed => state.percentage[0] = 100.,
                    process::Progress::Filling => state.percentage[1] += state.step[1],
                    process::Progress::Filled => {
                        state.percentage[1] = 0.;
                        state.percentage[2] += state.step[2];
                    }
                    process::Progress::Finished => {
                        info_dialog(state.i18n.info, state.i18n.saved_to_local.into());
                        state.percentage[1] = 100.;
                        state.percentage[2] = 100.;
                        state.process = None;
                        state.result_preview =
                            path2handle(&state.result_path, state.target_type == TargetType::Video);
                    }
                    process::Progress::Error => error_dialog(state.i18n.error, "".into()),
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

        let header = Row::new()
            .spacing(spacings::_3)
            .push(Text::new(title).size(spacings::_12).width(Length::Fill))
            .push(
                rou_btn(
                    i18n_btn,
                    state.i18n.symbol,
                    spacings::_12,
                    state.theme.transparency_btn(),
                )
                .on_press(Message::I18nPressed),
            )
            .push(
                rou_btn(
                    theme_btn,
                    state.theme.symbol(),
                    spacings::_12,
                    state.theme.transparency_btn(),
                )
                .on_press(Message::ThemePressed),
            )
            .push(
                rou_btn(
                    exit_btn,
                    "\u{f655}",
                    spacings::_12,
                    state.theme.transparency_btn(),
                )
                .on_press(Message::ExitPressed),
            );

        let back_l = format!("\u{f640} {}", state.i18n.back);
        let next_l = format!("{} \u{f641}", state.i18n.next);
        let btn_w = spacings::_24;
        let control_items: [Element<_>; 3] = match (steps.can_back(state), steps.can_next(state)) {
            (true, true) => [
                sec_btn(back_btn, back_l, btn_w, &state.theme)
                    .on_press(Message::BackPressed)
                    .into(),
                Space::with_width(Length::Units(10)).into(),
                pri_btn(next_btn, next_l, btn_w, &state.theme)
                    .on_press(Message::NextPressed)
                    .into(),
            ],

            (true, false) => [
                sec_btn(back_btn, back_l, btn_w, &state.theme)
                    .on_press(Message::BackPressed)
                    .into(),
                Space::with_width(Length::Units(10)).into(),
                Space::with_width(Length::Units(btn_w)).into(),
            ],

            (false, true) => [
                Space::with_width(Length::Units(10)).into(),
                Space::with_width(Length::Units(btn_w)).into(),
                pri_btn(next_btn, next_l, btn_w, &state.theme)
                    .on_press(Message::NextPressed)
                    .into(),
            ],

            (false, false) => [
                Space::with_width(Length::Units(0)).into(),
                Space::with_height(Length::Units(spacings::_11)).into(),
                Space::with_width(Length::Units(0)).into(),
            ],
        };
        let controls = control_items
            .into_iter()
            .fold(Row::new(), |row, items| row.push(items));

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
            .set_description(self.state.i18n.exit_desc)
            .set_buttons(MessageButtons::YesNo)
            .show()
        {
            self.should_exit = true;
        }
    }
}

fn path2handle(path: &Path, video: bool) -> Option<Handle> {
    if video {
        match first_frame(path) {
            Ok((width, height, pixels)) => Some(Handle::from_pixels(width, height, pixels)),
            Err(_) => None,
        }
    } else {
        match image::open(path) {
            Ok(img) => {
                let img = img.into_bgra8();
                let (width, height) = img.dimensions();
                Some(Handle::from_pixels(width, height, img.into_raw()))
            }
            Err(_) => None,
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
