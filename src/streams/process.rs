use {
    async_std::task::JoinHandle,
    iced::{
        futures::stream::{unfold, BoxStream},
        Subscription,
    },
    iced_native::subscription,
    image::RgbImage,
    image_diff::{LibItem, Mask, ProcessConfig, ProcessWrapper},
    std::{
        any::TypeId,
        hash::{Hash, Hasher},
        path::PathBuf,
        sync::Arc,
        vec::IntoIter,
    },
};

#[derive(Debug, Clone)]
pub struct Process {
    inner: Option<(ProcessConfig, String, String, bool, Arc<Vec<PathBuf>>)>,
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
            inner: Some((config, input, output, video, Arc::new(library))),
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
                        let (proc, (cnt, width, height)) =
                            ProcessWrapper::new(config, input, output, video);

                        let size = config.size as u32;
                        (
                            Progress::Started(
                                library.len() as f32,
                                ((width / size + 1) * (height / size + 1)) as f32,
                                cnt as f32,
                            ),
                            State::Start(proc, library),
                        )
                    }),

                    State::Start(proc, library) => Some({
                        let lib = Vec::with_capacity(library.len());
                        let tasks = proc.index(library.to_vec()).into_iter();
                        (Progress::None, State::Indexing(proc, tasks, lib))
                    }),

                    State::Indexing(mut proc, mut tasks, mut lib) => Some(match tasks.next() {
                        Some(task) => {
                            if let Some(i) = task.await {
                                lib.push(i);
                            }
                            (Progress::Indexing, State::Indexing(proc, tasks, lib))
                        }
                        None => match lib.is_empty() {
                            true => (Progress::Error, State::Finished),
                            false => {
                                proc.pre_fill(Arc::new(lib));
                                let tasks = proc.fill().unwrap().into_iter();
                                (Progress::Indexed, State::Filling(proc, tasks))
                            }
                        },
                    }),

                    State::Filling(mut proc, mut tasks) => Some(match tasks.next() {
                        Some(task) => {
                            let (mask, replace) = task.await;
                            proc.post_fill_step(mask, replace);
                            (Progress::Filling, State::Filling(proc, tasks))
                        }
                        None => {
                            proc.post_fill();
                            match proc.fill() {
                                Some(tasks) => {
                                    let tasks = tasks.into_iter();
                                    (Progress::Filled, State::Filling(proc, tasks))
                                }
                                None => (Progress::Finished, State::Finished),
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
    Ready(ProcessConfig, String, String, bool, Arc<Vec<PathBuf>>),
    Start(ProcessWrapper, Arc<Vec<PathBuf>>),
    Indexing(
        ProcessWrapper,
        IntoIter<JoinHandle<Option<LibItem>>>,
        Vec<LibItem>,
    ),
    Filling(ProcessWrapper, IntoIter<JoinHandle<(Mask, Arc<RgbImage>)>>),
    Finished,
}
