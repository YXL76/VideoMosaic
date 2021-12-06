use {
    argh::FromArgs,
    async_std::task::block_on,
    indicatif::{MultiProgress, ProgressBar, ProgressStyle},
    std::{
        ffi::OsStr,
        fs::{create_dir, read_dir},
        path::PathBuf,
    },
    video_mosaic_crawler::{download_urls, gen_client, get_urls},
    video_mosaic_diff::{
        CalculationUnit, ColorSpace, DistanceAlgorithm, Filter, ProcessConfig, ProcessWrapper,
        IMAGE_FILTER, VIDEO_FILTER,
    },
};

// const AUTHORS: &'static str = env!("CARGO_PKG_AUTHORS");
// const VERSION: &'static str = env!("CARGO_PKG_VERSION");

#[macro_export]
macro_rules! cli_args {
    ($name:ident; $(; $sub:expr)*) => {
        #[derive($crate::FromArgs, PartialEq)]
        /// Mosaic Video
        $( $sub
        )*
        struct $name {
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
    };
}

cli_args!(Opts;);

fn main() {
    let opts: Opts = argh::from_env();

    if opts.library.is_empty() && opts.keyword.is_empty() {
        panic!(
            r#"The following required arguments were not provided:
    --keyword <KEYWORD>...
    --library <LIBRARY>..."#
        )
    }

    let video = {
        let ext = opts.target.extension().unwrap().to_str().unwrap();
        if IMAGE_FILTER.contains(&ext) {
            false
        } else if VIDEO_FILTER.contains(&ext) {
            video_mosaic_diff::init();
            true
        } else {
            panic!("Target is not supported!");
        }
    };

    let mut library = Vec::with_capacity(opts.keyword.len() * opts.num);
    for lib in opts.library {
        push_file_to_lib(&mut library, &lib);
    }

    if !opts.keyword.is_empty() {
        println!("Crawling images:");
        let client = gen_client();

        for keyword in opts.keyword {
            let mut folder = PathBuf::new();
            for i in 0.. {
                folder = PathBuf::from(format!("{}-{}", keyword, i));
                if !folder.exists() {
                    break;
                }
            }
            create_dir(&folder).unwrap();

            block_on(async {
                let (num, tasks) = get_urls(client.clone(), keyword.clone(), opts.num)
                    .await
                    .unwrap();
                let mut urls = Vec::with_capacity(num);
                for task in tasks {
                    if let Ok(ret) = task.await {
                        urls.extend_from_slice(&ret);
                    }
                }

                let pb = gen_progress_bar(keyword.as_str(), num as u64);

                let tasks = download_urls(client.clone(), urls, folder.clone());
                for task in tasks {
                    let _ = task.await;
                    pb.inc(1);
                }
                pb.finish();
            });

            push_file_to_lib(&mut library, &folder);
        }
    }

    println!("Processing image:");

    let config = ProcessConfig {
        size: opts.size,
        k: opts.k,
        hamerly: opts.hamerly,
        calc_unit: opts.calc_unit,
        color_space: opts.color_space,
        dist_algo: opts.dist_algo,
        filter: opts.filter,
        quad_iter: opts.quad_iter,
        overlay: opts.overlay,
    };

    let ext = OsStr::new(if video { "mp4" } else { "png" });
    let mut path = opts.target.clone();
    let mut base = opts.target.file_stem().unwrap().to_os_string();
    base.push("-mosaic");
    path.set_file_name(&base);
    path.set_extension(ext);
    while path.exists() {
        base.push("_");
        path.set_file_name(&base);
        path.set_extension(ext);
    }

    let mut proc = ProcessWrapper::new(
        config,
        opts.target.to_string_lossy().to_string(),
        path.to_string_lossy().to_string(),
        video,
    );

    let index = gen_progress_bar("Index", library.len() as u64);
    let m = MultiProgress::new();
    let fill = m.add(gen_progress_bar(
        "Fill",
        (proc.width() as u64 / opts.size as u64 + 1)
            * (proc.height() as u64 / opts.size as u64 + 1),
    ));
    let total = m.add(gen_progress_bar("Total", proc.frames() as u64));

    block_on(async move {
        let mut lib_color = Vec::with_capacity(library.len());
        let mut lib_image = Vec::with_capacity(library.len());
        let tasks = proc.index(library);
        for task in tasks {
            if let Some((color, image)) = task.await {
                lib_color.push(color);
                lib_image.push(image);
            }
            index.inc(1);
        }
        proc.post_index(lib_color, lib_image);
        index.finish();

        while proc.pre_fill() {
            fill.reset();
            let tasks = proc.fill();
            for task in tasks {
                let (mask, replace_idx) = task.await;
                proc.post_fill_step(mask, replace_idx);
                fill.inc(1);
            }
            proc.post_fill();
            fill.finish();
            total.inc(1);
        }
        total.finish();
    });
}

fn push_file_to_lib(library: &mut Vec<PathBuf>, folder: &PathBuf) {
    for entry in read_dir(folder).unwrap() {
        if let Ok(entry) = entry {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let ext = path
                .extension()
                .unwrap_or_default()
                .to_str()
                .unwrap_or_default();
            if IMAGE_FILTER.contains(&ext) {
                library.push(path)
            }
        }
    }
}

fn gen_progress_bar(title: &str, total: u64) -> ProgressBar {
    let pb = ProgressBar::new(total);
    let template = format!("{:<7}", title)
        + "[{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos:>7}/{len:7} {eta:>7}";
    pb.set_style(
        ProgressStyle::default_bar()
            .template(template.as_str())
            .with_key("eta", |state| {
                format!("({:.1}s)", state.eta().as_secs_f64())
            })
            .progress_chars("#>-"),
    );
    pb
}

fn str2cu(cu: &str) -> Result<CalculationUnit, String> {
    match cu {
        "average" => Ok(CalculationUnit::Average),
        "pixel" => Ok(CalculationUnit::Pixel),
        "k_means" => Ok(CalculationUnit::KMeans),
        _ => Err("incorrect calculation unit".into()),
    }
}

fn str2cs(cs: &str) -> Result<ColorSpace, String> {
    match cs {
        "rgb" => Ok(ColorSpace::RGB),
        "hsv" => Ok(ColorSpace::HSV),
        "cielab" => Ok(ColorSpace::CIELAB),
        _ => Err("incorrect color space".into()),
    }
}

fn str2da(da: &str) -> Result<DistanceAlgorithm, String> {
    match da {
        "euclidean" => Ok(DistanceAlgorithm::Euclidean),
        "ciede2000" => Ok(DistanceAlgorithm::CIEDE2000),
        _ => Err("incorrect distance algorithm".into()),
    }
}

fn str2filter(filter: &str) -> Result<Filter, String> {
    match filter {
        "nearest" => Ok(Filter::Nearest),
        "triangle" => Ok(Filter::Triangle),
        "catmullRom" => Ok(Filter::CatmullRom),
        "gaussian" => Ok(Filter::Gaussian),
        "lanczos3" => Ok(Filter::Lanczos3),
        _ => Err("incorrect filter".into()),
    }
}
