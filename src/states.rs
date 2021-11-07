use {
    crossbeam_utils::atomic::AtomicCell,
    std::{lazy::SyncLazy, path::PathBuf, sync::RwLock},
};

#[derive(Default)]
pub struct State {
    pub target_type: TargetType,
    pub target_path: PathBuf,
}

pub static STATE: SyncLazy<RwLock<State>> = SyncLazy::new(|| RwLock::new(State::default()));

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

pub static I18N: AtomicCell<&I18n> = AtomicCell::new(&EN);

pub struct I18n {
    pub symbol: &'static str,

    pub back: &'static str,
    pub next: &'static str,

    pub choose_target: &'static str,
    pub choose_image: &'static str,
    pub choose_video: &'static str,
}

pub fn toggle_i18n() {
    match {
        let i18n = I18N.load();
        i18n.symbol
    } {
        "En" => I18N.store(&ZH_CN),
        "中" => I18N.store(&EN),
        _ => (),
    }
}

const EN: I18n = I18n {
    symbol: "En",

    back: " Back",
    next: "Next ",

    choose_target: "Choose Target",
    choose_image: " Choose Image",
    choose_video: " Choose Video",
};

const ZH_CN: I18n = I18n {
    symbol: "中",

    back: " 后退",
    next: "前进 ",

    choose_target: "选择目标",
    choose_image: " 选择图片",
    choose_video: " 选择视频",
};
