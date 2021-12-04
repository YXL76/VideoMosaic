use {
    iced::{
        futures::stream::{unfold, BoxStream},
        Subscription,
    },
    iced_native::subscription,
    image::RgbImage,
    mosaic_video_diff::{LibItem, Mask, ProcessConfig, ProcessWrapper, RawColor, TasksIter},
    std::{
        any::TypeId,
        hash::{Hash, Hasher},
        path::PathBuf,
    },
};

type ProcessData = (ProcessConfig, String, String, bool, Vec<PathBuf>);

#[derive(Debug, Clone)]
pub struct Process {
    inner: Option<ProcessData>,
}

impl Process {
    #[inline(always)]
    pub fn new(
        config: ProcessConfig,
        input: String,
        output: String,
        video: bool,
        library: Vec<PathBuf>,
    ) -> Self {
        Self {
            inner: Some((config, input, output, video, library)),
        }
    }

    #[inline(always)]
    pub fn subscription(&self) -> Subscription<Progress> {
        Subscription::from_recipe(self.clone())
    }
    #[inline(always)]
    pub fn clear(&mut self) {
        self.inner = None;
    }
}

impl<H, E> subscription::Recipe<H, E> for Process
where
    H: Hasher,
{
    type Output = Progress;

    fn hash(&self, state: &mut H) {
        struct Marker;
        TypeId::of::<Marker>().hash(state);
        0.hash(state);
    }

    fn stream(self: Box<Self>, _input: BoxStream<'static, E>) -> BoxStream<'static, Self::Output> {
        let (config, input, output, video, library) = self.inner.unwrap();
        Box::pin(unfold(
            State::Ready(config, input, output, video, library),
            move |state| async move {
                match state {
                    State::Ready(config, input, output, video, library) => Some({
                        let proc = ProcessWrapper::new(config, input, output, video);
                        let size = config.size as u32;
                        (
                            Progress::Started(
                                library.len() as f32,
                                ((proc.width() / size + 1) * (proc.height() / size + 1)) as f32,
                                proc.frames() as f32,
                            ),
                            State::Start(proc, library),
                        )
                    }),

                    State::Start(proc, library) => Some({
                        let lib_color = Vec::with_capacity(library.len());
                        let lib_image = Vec::with_capacity(library.len());
                        let tasks = proc.index(library).into_iter();
                        (
                            Progress::None,
                            State::Indexing(proc, tasks, lib_color, lib_image),
                        )
                    }),

                    State::Indexing(mut proc, mut tasks, mut lib_color, mut lib_image) => {
                        Some(match tasks.next() {
                            Some(task) => {
                                if let Some((color, image)) = task.await {
                                    lib_color.push(color);
                                    lib_image.push(image);
                                }
                                (
                                    Progress::Indexing,
                                    State::Indexing(proc, tasks, lib_color, lib_image),
                                )
                            }
                            None => match lib_image.is_empty() {
                                true => (Progress::Error, State::Finished),
                                false => {
                                    proc.post_index(lib_color, lib_image);
                                    let _ = proc.pre_fill();
                                    let tasks = proc.fill().into_iter();
                                    (Progress::Indexed, State::Filling(proc, tasks))
                                }
                            },
                        })
                    }

                    State::Filling(mut proc, mut tasks) => Some(match tasks.next() {
                        Some(task) => {
                            let (mask, replace_idx) = task.await;
                            proc.post_fill_step(mask, replace_idx);
                            (Progress::Filling, State::Filling(proc, tasks))
                        }
                        None => {
                            proc.post_fill();
                            match proc.pre_fill() {
                                true => {
                                    let tasks = proc.fill().into_iter();
                                    (Progress::Filled, State::Filling(proc, tasks))
                                }
                                false => (Progress::Finished, State::Finished),
                            }
                        }
                    }),

                    State::Finished => None,
                }
            },
        ))
    }
}

#[derive(Debug, Clone)]
pub enum Progress {
    Started(f32, f32, f32),
    Indexing,
    Indexed,
    Filling,
    Filled,
    Finished,
    Error,
    None,
}

#[derive(Debug)]
enum State {
    Ready(ProcessConfig, String, String, bool, Vec<PathBuf>),
    Start(ProcessWrapper, Vec<PathBuf>),
    Indexing(
        ProcessWrapper,
        TasksIter<Option<LibItem>>,
        Vec<Vec<RawColor>>,
        Vec<RgbImage>,
    ),
    Filling(ProcessWrapper, TasksIter<(Mask, usize)>),
    Finished,
}
