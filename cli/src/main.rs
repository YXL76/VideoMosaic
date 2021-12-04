use {
    async_std::task::block_on,
    clap::Parser,
    indicatif::{MultiProgress, ProgressBar, ProgressStyle},
    mosaic_video_crawler::{download_urls, gen_client, get_urls},
    mosaic_video_diff::{
        CalculationUnit, ColorSpace, DistanceAlgorithm, Filter, ProcessConfig, ProcessWrapper,
        IMAGE_FILTER, VIDEO_FILTER,
    },
    std::{
        ffi::OsStr,
        fs::{create_dir, read_dir},
        path::PathBuf,
        sync::Arc,
    },
};

/// Mosaic Video CLI
#[derive(Parser)]
#[clap(version = "0.1", author = "YXL76 <chenxin.lan.76@gmail.com>")]
struct Opts {
    /// The path of the target file
    #[clap(parse(from_os_str))]
    target: PathBuf,
    /// Keywords to crawl the images
    #[clap(short, long, required_unless_present = "library")]
    keyword: Vec<String>,
    /// The number of images that need to be crawled
    #[clap(short, long, default_value = "100")]
    num: usize,
    /// The path of the libraries
    #[clap(short, long, parse(from_os_str), required_unless_present = "keyword")]
    library: Vec<PathBuf>,
    /// The size of the block
    #[clap(short, long, default_value = "5")]
    size: u16,
    /// K-means (k)
    #[clap(long, default_value = "1")]
    k: u8,
    /// Use Hamerly’s K-Means Clustering Algorithm
    #[clap(short, long)]
    hamerly: bool,
    /// Calculation unit (average, pixel, k_means)
    #[clap(long, default_value = "average", parse(from_str = str2cu))]
    calc_unit: CalculationUnit,
    /// Color space (rgb, hsv, cielab)
    #[clap(long, default_value = "cielab", parse(from_str = str2cs))]
    color_space: ColorSpace,
    /// Distance algorithm (euclidean, ciede2000)
    #[clap(long, default_value = "ciede2000", parse(from_str = str2da))]
    dist_algo: DistanceAlgorithm,
    /// Filter (nearest, triangle, catmullRom, gaussian, lanczos3)
    #[clap(long, default_value = "nearest", parse(from_str = str2filter))]
    filter: Filter,
    /// The number of iterations of the quadrant
    #[clap(long)]
    quad_iter: Option<usize>,
    /// Overlay image and set the bottom image's alpha channel
    #[clap(long)]
    overlay: Option<u8>,
}

fn main() {
    let opts: Opts = Opts::parse();

    let video = {
        let ext = opts.target.extension().unwrap().to_str().unwrap();
        if IMAGE_FILTER.contains(&ext) {
            false
        } else if VIDEO_FILTER.contains(&ext) {
            mosaic_video_diff::init();
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
        let client = Arc::new(gen_client());

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
        let mut lib = Vec::with_capacity(library.len());
        let tasks = proc.index(library);
        for task in tasks {
            if let Some(i) = task.await {
                lib.push(i);
            }
            index.inc(1);
        }
        proc.post_index(Arc::new(lib));
        index.finish();

        while proc.pre_fill() {
            fill.reset();
            let tasks = proc.fill();
            for task in tasks {
                let (mask, replace) = task.await;
                proc.post_fill_step(mask, replace);
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

fn str2cu(cu: &str) -> CalculationUnit {
    match cu {
        "average" => CalculationUnit::Average,
        "pixel" => CalculationUnit::Pixel,
        "k_means" => CalculationUnit::KMeans,
        _ => Default::default(),
    }
}

fn str2cs(cs: &str) -> ColorSpace {
    match cs {
        "rgb" => ColorSpace::RGB,
        "hsv" => ColorSpace::HSV,
        "cielab" => ColorSpace::CIELAB,
        _ => Default::default(),
    }
}

fn str2da(da: &str) -> DistanceAlgorithm {
    match da {
        "euclidean" => DistanceAlgorithm::Euclidean,
        "ciede2000" => DistanceAlgorithm::CIEDE2000,
        _ => Default::default(),
    }
}

fn str2filter(filter: &str) -> Filter {
    match filter {
        "nearest" => Filter::Nearest,
        "triangle" => Filter::Triangle,
        "catmullRom" => Filter::CatmullRom,
        "gaussian" => Filter::Gaussian,
        "lanczos3" => Filter::Lanczos3,
        _ => Default::default(),
    }
}
