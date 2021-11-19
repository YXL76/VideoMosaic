use {
    async_std::task::JoinHandle,
    iced::{
        futures::stream::{unfold, BoxStream},
        Subscription,
    },
    iced_native::subscription,
    image::{ImageBuffer, RgbImage},
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
    config: ProcessConfig,
    img: Arc<RgbImage>,
    library: Arc<Vec<PathBuf>>,
    masks: Arc<Vec<Mask>>,
}

impl Process {
    #[inline(always)]
    pub fn new(
        config: ProcessConfig,
        img: Arc<RgbImage>,
        library: Arc<Vec<PathBuf>>,
        masks: Arc<Vec<Mask>>,
    ) -> Self {
        Self {
            config,
            img,
            library,
            masks,
        }
    }

    #[inline(always)]
    pub fn subscription(&self) -> Subscription<Progress> {
        Subscription::from_recipe(self.clone())
    }
}

impl<H, E> subscription::Recipe<H, E> for Process
where
    H: Hasher,
{
    type Output = Progress;

    fn hash(&self, state: &mut H) {
        TypeId::of::<Self>().hash(state);
        0.hash(state);
    }

    fn stream(self: Box<Self>, _input: BoxStream<'static, E>) -> BoxStream<'static, Self::Output> {
        Box::pin(unfold(State::Ready(self), move |state| async move {
            match state {
                State::Ready(s) => Some({
                    let proc = ProcessWrapper::new(s.config);
                    let lib = Vec::with_capacity(s.library.len());
                    let tasks = proc.index(&s.library).into_iter();
                    (
                        Progress::None,
                        State::Indexing(s.img, s.masks, proc, tasks, lib),
                    )
                }),

                State::Indexing(img, masks, proc, mut tasks, mut lib) => Some(match tasks.next() {
                    Some(task) => {
                        if let Some(i) = task.await {
                            lib.push(i);
                        }
                        (
                            Progress::Indexing,
                            State::Indexing(img, masks, proc, tasks, lib),
                        )
                    }
                    None => match lib.is_empty() {
                        true => (
                            Progress::Error(String::from("Library is empty.")),
                            State::Finished,
                        ),
                        false => {
                            let (width, height) = img.dimensions();
                            let img_buf: RgbImage = ImageBuffer::new(width, height);
                            let tasks = proc.fill(img, Arc::new(lib), &masks).into_iter();
                            (Progress::None, State::Filling(proc, tasks, img_buf))
                        }
                    },
                }),

                State::Filling(proc, mut tasks, mut img_buf) => Some(match tasks.next() {
                    Some(task) => {
                        let ((x, y, w, h), replace) = task.await;
                        for j in 0..h {
                            for i in 0..w {
                                img_buf.put_pixel(i + x, j + y, *replace.get_pixel(i, j));
                            }
                        }
                        (Progress::Filling, State::Filling(proc, tasks, img_buf))
                    }
                    None => (Progress::Finished(img_buf), State::Finished),
                }),

                State::Finished => None,
            }
        }))
    }
}

#[derive(Debug, Clone)]
pub enum Progress {
    Indexing,
    Filling,
    Finished(RgbImage),
    None,
    Error(String),
}

#[derive(Debug)]
enum State {
    Ready(Box<Process>),
    Indexing(
        Arc<RgbImage>,
        Arc<Vec<Mask>>,
        ProcessWrapper,
        IntoIter<JoinHandle<Option<LibItem>>>,
        Vec<LibItem>,
    ),
    Filling(
        ProcessWrapper,
        IntoIter<JoinHandle<(Mask, Arc<RgbImage>)>>,
        RgbImage,
    ),
    Finished,
}
