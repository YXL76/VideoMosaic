use {
    crate::styles::Theme,
    std::{collections::HashMap, path::PathBuf},
};

#[derive(Default)]
pub struct State {
    pub i18n: &'static I18n,
    pub theme: Theme,
    pub target_type: TargetType,
    pub target_path: PathBuf,
    pub libraries: HashMap<PathBuf, Vec<PathBuf>>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
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

    pub choose_library: &'static str,
    pub delete: &'static str,
    pub delete_desc: &'static str,
}

impl Default for &I18n {
    fn default() -> Self {
        &EN
    }
}

pub const EN: I18n = I18n {
    symbol: "En",

    back: "Back",
    next: "Next",

    choose_target: "Choose Target",
    choose_image: "Choose Image",
    choose_video: "Choose Video",

    choose_library: "Choose Library",
    delete: "Delete",
    delete_desc: "Are you sure to delete?",
};

pub const ZH_CN: I18n = I18n {
    symbol: "中",

    back: "后退",
    next: "前进",

    choose_target: "选择目标",
    choose_image: "选择图片",
    choose_video: "选择视频",

    choose_library: "选择图片库",
    delete: "删除",
    delete_desc: "你确定要删除吗？",
};
