use argh::FromArgs;

#[derive(FromArgs, PartialEq)]
#[argh(description = "Video Mosaic GUI.")]
struct Opts {
    /// if enabled, spread text workload in multiple threads when multiple cores are available.
    /// By default, it is disabled.
    #[argh(switch, short = 't')]
    text_multithreading: bool,

    /// if set to true, the renderer will try to perform antialiasing for some primitives.
    #[argh(switch, short = 'a')]
    antialiasing: bool,
}

fn main() {
    let Opts {
        text_multithreading,
        antialiasing,
    } = argh::from_env();
    video_mosaic_gui::run(text_multithreading, antialiasing).unwrap()
}
