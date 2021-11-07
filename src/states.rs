use std::path::PathBuf;

#[derive(Default)]
pub struct State {
    pub i18n: &'static I18n,
    pub target_type: TargetType,
    pub target_path: PathBuf,
}

#[derive(Debug, Clone, Copy)]
pub enum TargetType {
    None,
    Image,
    Video,
}

impl Default for TargetType {
    fn default() -> Self {
        TargetType::None
    }
}

pub struct I18n {
    pub symbol: &'static str,

    pub back: &'static str,
    pub next: &'static str,

    pub choose_target: &'static str,
    pub choose_image: &'static str,
    pub choose_video: &'static str,
}

impl Default for &I18n {
    fn default() -> Self {
        &EN
    }
}

pub const EN: I18n = I18n {
    symbol: "En",

    back: " Back",
    next: "Next ",

    choose_target: "Choose Target",
    choose_image: " Choose Image",
    choose_video: " Choose Video",
};

pub const ZH_CN: I18n = I18n {
    symbol: "中",

    back: " 后退",
    next: "前进 ",

    choose_target: "选择目标",
    choose_image: " 选择图片",
    choose_video: " 选择视频",
};
