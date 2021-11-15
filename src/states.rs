use {
    crate::{steps::TargetType, styles::Theme},
    image_diff::{CalculationUnit, ColorSpace, DistanceAlgorithm},
    std::{collections::HashMap, path::PathBuf},
};

pub const LIBRARY_BTN_CNT: usize = 16;

#[derive(Default)]
pub struct State {
    pub i18n: &'static I18n,
    pub theme: Theme,
    pub target_type: TargetType,
    pub target_path: PathBuf,
    pub libraries: HashMap<PathBuf, Vec<PathBuf>>,
    pub calc_unit: CalculationUnit,
    pub color_space: ColorSpace,
    pub dist_algo: DistanceAlgorithm,
}

pub struct I18n {
    pub symbol: &'static str,

    pub back: &'static str,
    pub next: &'static str,

    pub choose_target: &'static str,
    pub choose_image: &'static str,
    pub choose_video: &'static str,

    pub choose_library: &'static str,
    pub from_the_web: &'static str,
    pub delete: &'static str,
    pub delete_desc: &'static str,

    pub choose_method: &'static str,
    pub calc_unit: &'static str,
    pub calc_unit_average: &'static str,
    pub calc_unit_pixel: &'static str,
    pub calc_unit_k_means: &'static str,
    pub color_space: &'static str,
    pub color_space_rgb: &'static str,
    pub color_space_hsv: &'static str,
    pub color_space_cielab: &'static str,
    pub dist_algo: &'static str,
    pub dist_algo_euclidean: &'static str,
    pub dist_algo_ciede2000: &'static str,

    pub process: &'static str,
    pub start: &'static str,
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
    from_the_web: "From the Web",
    delete: "Delete",
    delete_desc: "Are you sure to delete?",

    choose_method: "Choose Method",
    calc_unit: "Calculation Unit",
    calc_unit_average: "Average",
    calc_unit_pixel: "Pixel",
    calc_unit_k_means: "K-means",
    color_space: "Color Space",
    color_space_rgb: "RGB",
    color_space_hsv: "HSV",
    color_space_cielab: "CIE L*a*b*",
    dist_algo: "Distance Algorithm",
    dist_algo_euclidean: "Euclidean",
    dist_algo_ciede2000: "CIEDE2000",

    process: "Process",
    start: "Start",
};

pub const ZH_CN: I18n = I18n {
    symbol: "中",

    back: "后退",
    next: "前进",

    choose_target: "选择目标",
    from_the_web: "来自网络",
    choose_image: "选择图片",
    choose_video: "选择视频",

    choose_library: "选择图片库",
    delete: "删除",
    delete_desc: "你确定要删除吗？",

    choose_method: "选择方案",
    calc_unit: "计算单位",
    calc_unit_average: "均值",
    calc_unit_pixel: "像素",
    calc_unit_k_means: "K-means",
    color_space: "颜色空间",
    color_space_rgb: "RGB",
    color_space_hsv: "HSV",
    color_space_cielab: "CIE L*a*b*",
    dist_algo: "距离算法",
    dist_algo_euclidean: "欧几里德",
    dist_algo_ciede2000: "CIEDE2000",

    process: "处理",
    start: "开始",
};
