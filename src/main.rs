use {
    argh::FromArgs,
    std::path::PathBuf,
    video_mosaic_diff::{
        str2cs, str2cu, str2da, str2filter, CalculationUnit, ColorSpace, DistanceAlgorithm, Filter,
        ProcessConfig,
    },
};

// const AUTHORS: &'static str = env!("CARGO_PKG_AUTHORS");
// const VERSION: &'static str = env!("CARGO_PKG_VERSION");

#[derive(FromArgs, PartialEq)]
#[argh(description = "Video Mosaic.")]
struct Opts {
    /// if enabled, spread text workload in multiple threads when multiple cores are available.
    /// By default, it is disabled.
    #[argh(switch, short = 't')]
    text_multithreading: bool,

    /// if set to true, the renderer will try to perform antialiasing for some primitives.
    #[argh(switch, short = 'a')]
    antialiasing: bool,

    #[argh(subcommand)]
    nested: Option<SubCommandEnum>,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand)]
enum SubCommandEnum {
    Cli(SubCommandCli),
}

#[derive(FromArgs, PartialEq, Debug)]
/// CLI subcommand.
#[argh(subcommand, name = "cli")]
struct SubCommandCli {
    /// the path of the target file
    #[argh(positional)]
    target: PathBuf,
    /// keywords to crawl the images
    #[argh(option, short = 'k')]
    keyword: Vec<String>,
    /// the number of images that need to be crawled
    #[argh(option, short = 'n', default = "100")]
    num: usize,
    /// the path of the libraries
    #[argh(option, short = 'l')]
    library: Vec<PathBuf>,
    /// the size of the block
    #[argh(option, short = 's', default = "50")]
    size: u16,
    /// k-means (k)
    #[argh(option, default = "1")]
    k: u8,
    /// use Hamerly’s K-Means Clustering Algorithm
    #[argh(switch, short = 'h')]
    hamerly: bool,
    /// calculation unit (average, pixel, k_means)
    #[argh(option, default = "CalculationUnit::default()", from_str_fn(str2cu))]
    calc_unit: CalculationUnit,
    /// color space (rgb, hsv, cielab)
    #[argh(option, default = "ColorSpace::default()", from_str_fn(str2cs))]
    color_space: ColorSpace,
    /// distance algorithm (euclidean, ciede2000)
    #[argh(option, default = "DistanceAlgorithm::default()", from_str_fn(str2da))]
    dist_algo: DistanceAlgorithm,
    /// filter (nearest, triangle, catmullRom, gaussian, lanczos3)
    #[argh(option, default = "Filter::default()", from_str_fn(str2filter))]
    filter: Filter,
    /// the number of iterations of the quadrant
    #[argh(option)]
    quad_iter: Option<usize>,
    /// overlay image and set the bottom image's alpha channel
    #[argh(option)]
    overlay: Option<u8>,
}

fn main() {
    let Opts {
        text_multithreading,
        antialiasing,
        nested,
    } = argh::from_env();
    if let Some(SubCommandEnum::Cli(opts)) = nested {
        let SubCommandCli {
            target,
            keyword,
            num,
            library,
            size,
            k,
            hamerly,
            calc_unit,
            color_space,
            dist_algo,
            filter,
            quad_iter,
            overlay,
        } = opts;

        let config = ProcessConfig {
            size,
            k,
            hamerly,
            calc_unit,
            color_space,
            dist_algo,
            filter,
            quad_iter,
            overlay,
        };

        video_mosaic_cli::run(target, keyword, num, library, config);
    } else {
        video_mosaic_gui::run(text_multithreading, antialiasing).unwrap()
    }
}
