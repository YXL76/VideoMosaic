use {
    async_std::task::block_on,
    indicatif::{MultiProgress, ProgressBar, ProgressStyle},
    std::{
        ffi::OsStr,
        fs::{create_dir, read_dir},
        path::{Path, PathBuf},
    },
    video_mosaic_crawler::{download_urls, gen_client, get_urls},
    video_mosaic_diff::{ProcessConfig, ProcessWrapper, IMAGE_FILTER, VIDEO_FILTER},
};

pub fn run(
    target: PathBuf,
    keyword: Vec<String>,
    num: usize,
    library: Vec<PathBuf>,
    config: ProcessConfig,
) {
    if library.is_empty() && keyword.is_empty() {
        panic!(
            r#"The following required arguments were not provided:
    --keyword <KEYWORD>...
    --library <LIBRARY>..."#
        )
    }

    let video = {
        let ext = target.extension().unwrap().to_str().unwrap();
        if IMAGE_FILTER.contains(&ext) {
            false
        } else if VIDEO_FILTER.contains(&ext) {
            video_mosaic_diff::init();
            true
        } else {
            panic!("Target is not supported!");
        }
    };

    let mut libraries = Vec::with_capacity(keyword.len() * num);
    for lib in library {
        push_file_to_lib(&mut libraries, &lib);
    }

    if !keyword.is_empty() {
        println!("Crawling images:");
        let client = gen_client();

        for keyword in keyword {
            let mut folder = PathBuf::new();
            for i in 0.. {
                folder = PathBuf::from(format!("{}-{}", keyword, i));
                if !folder.exists() {
                    break;
                }
            }
            create_dir(&folder).unwrap();

            block_on(async {
                let (num, tasks) = get_urls(client.clone(), keyword.clone(), num)
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

            push_file_to_lib(&mut libraries, &folder);
        }
    }

    println!("Processing image:");

    let ext = OsStr::new(if video { "mp4" } else { "png" });
    let mut path = target.clone();
    let mut base = target.file_stem().unwrap().to_os_string();
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
        target.to_string_lossy().to_string(),
        path.to_string_lossy().to_string(),
        video,
    );

    let index = gen_progress_bar("Index", libraries.len() as u64);
    let m = MultiProgress::new();
    let fill = m.add(gen_progress_bar(
        "Fill",
        (proc.width() as u64 / config.size as u64 + 1)
            * (proc.height() as u64 / config.size as u64 + 1),
    ));
    let total = m.add(gen_progress_bar("Total", proc.frames() as u64));

    block_on(async move {
        let mut lib_color = Vec::with_capacity(libraries.len());
        let mut lib_image = Vec::with_capacity(libraries.len());
        let tasks = proc.index(libraries);
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

fn push_file_to_lib(library: &mut Vec<PathBuf>, folder: &Path) {
    for entry in read_dir(folder).unwrap().flatten() {
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
