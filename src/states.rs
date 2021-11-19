use {
    crate::{
        steps::TargetType,
        streams::{crawler, process},
        styles::Theme,
    },
    image::imageops::FilterType,
    image_diff::{CalculationUnit, ColorSpace, DistanceAlgorithm},
    std::{collections::HashMap, path::PathBuf},
};

pub const LIBRARY_BTN_CNT: usize = 16;
pub const IMAGE_FILTER: [&str; 3] = ["png", "jpg", "jpeg"];
pub const VIDEO_FILTER: [&str; 1] = ["mp4"];

pub struct State {
    pub i18n: &'static I18n,
    pub theme: Theme,

    pub target_type: TargetType,
    pub target_path: PathBuf,

    pub libraries: HashMap<PathBuf, Vec<PathBuf>>,
    pub pending: Vec<(String, String)>,
    pub crawler_id: usize,
    pub crawlers: HashMap<usize, crawler::Crawler>,

    pub calc_unit: CalculationUnit,
    pub color_space: ColorSpace,
    pub dist_algo: DistanceAlgorithm,
    pub filter: Filter,
    pub k: u8,
    pub hamerly: bool,
    pub size: u16,

    pub step: [f32; 3],
    pub percentage: [f32; 3],
    pub process: Option<process::Process>,
}

impl State {
    #[inline(always)]
    pub fn is_full(&self) -> bool {
        self.libraries.len() + self.pending.len() + self.crawlers.len() >= LIBRARY_BTN_CNT
    }
}

impl Default for State {
    fn default() -> Self {
        Self {
            i18n: Default::default(),
            theme: Default::default(),
            target_type: Default::default(),
            target_path: Default::default(),
            libraries: Default::default(),
            pending: Default::default(),
            crawler_id: Default::default(),
            crawlers: Default::default(),
            calc_unit: Default::default(),
            color_space: Default::default(),
            dist_algo: Default::default(),
            filter: Default::default(),
            k: 1,
            hamerly: Default::default(),
            size: 100,
            step: Default::default(),
            percentage: Default::default(),
            process: Default::default(),
        }
    }
}

pub struct I18n {
    pub symbol: &'static str,
    pub info: &'static str,
    pub error: &'static str,
    pub exit: &'static str,
    pub exit_desc: &'static str,

    pub back: &'static str,
    pub next: &'static str,

    pub choose_target: &'static str,
    pub choose_image: &'static str,
    pub choose_video: &'static str,

    pub choose_library: &'static str,
    pub from_the_web: &'static str,
    pub delete: &'static str,
    pub delete_desc: &'static str,
    pub keyword_prompt: &'static str,

    pub choose_method: &'static str,
    pub calc_unit: &'static str,
    pub average: &'static str,
    pub pixel: &'static str,
    pub k_means: &'static str,
    pub color_space: &'static str,
    pub dist_algo: &'static str,
    pub sampling: &'static str,
    pub nearest: &'static str,
    pub triangle: &'static str,
    pub catmull_rom: &'static str,
    pub gaussian: &'static str,
    pub lanczos3: &'static str,
    pub configuration: &'static str,
    pub size: &'static str,

    pub process: &'static str,
    pub start: &'static str,
    pub index: &'static str,
    pub fill: &'static str,
    pub composite: &'static str,
}

impl Default for &I18n {
    fn default() -> Self {
        &EN
    }
}

pub const EN: I18n = I18n {
    symbol: "En",
    info: "Info",
    error: "Error",
    exit: "Exit",
    exit_desc: "Are you sure to exit?",

    back: "Back",
    next: "Next",

    choose_target: "Choose Target",
    choose_image: "Choose Image",
    choose_video: "Choose Video",

    choose_library: "Choose Library",
    from_the_web: "From the Web",
    delete: "Delete",
    delete_desc: "Are you sure to delete?",
    keyword_prompt: "Please enter keyword.",

    choose_method: "Choose Method",
    calc_unit: "Calculation Unit",
    average: "Average",
    pixel: "Pixel",
    k_means: "K-means",
    color_space: "Color Space",
    dist_algo: "Distance Algorithm",
    sampling: "Sampling",
    nearest: "Nearest",
    triangle: "Triangle",
    catmull_rom: "Catmull Rom",
    gaussian: "Gaussian",
    lanczos3: "Lanczos3",
    configuration: "Configuration",
    size: "Size",

    process: "Process",
    start: "Start",
    index: "Index",
    fill: "Fill",
    composite: "Composite",
};

pub const ZH_CN: I18n = I18n {
    symbol: "中",
    info: "信息",
    error: "错误",
    exit: "退出",
    exit_desc: "你确定要退出吗？",

    back: "后退",
    next: "前进",

    choose_target: "选择目标",
    from_the_web: "来自网络",
    choose_image: "选择图片",
    choose_video: "选择视频",

    choose_library: "选择图片库",
    delete: "删除",
    delete_desc: "你确定要删除吗？",
    keyword_prompt: "请输入关键词",

    choose_method: "选择方案",
    calc_unit: "计算单位",
    average: "均值",
    pixel: "像素",
    k_means: "K-means",
    color_space: "颜色空间",
    dist_algo: "距离算法",
    sampling: "采样",
    nearest: "最近邻",
    triangle: "双立方/三角",
    catmull_rom: "Catmull Rom",
    gaussian: "高斯",
    lanczos3: "Lanczos 3",
    configuration: "配置",
    size: "大小",

    process: "处理",
    start: "开始",
    index: "建立索引",
    fill: "填充图片",
    composite: "合成",
};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Filter {
    Nearest,
    Triangle,
    CatmullRom,
    Gaussian,
    Lanczos3,
}

impl Default for Filter {
    fn default() -> Self {
        Self::Nearest
    }
}

impl Into<FilterType> for Filter {
    fn into(self) -> FilterType {
        match self {
            Self::Nearest => FilterType::Nearest,
            Self::Triangle => FilterType::Triangle,
            Self::CatmullRom => FilterType::CatmullRom,
            Self::Gaussian => FilterType::Gaussian,
            Self::Lanczos3 => FilterType::Lanczos3,
        }
    }
}
