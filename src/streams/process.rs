use {
    crate::states::Filter,
    iced::{
        futures::stream::{unfold, BoxStream},
        Subscription,
    },
    iced_native::subscription,
    image::{ImageBuffer, RgbImage},
    image_diff::{
        CalculationUnit, ColorSpace, DistanceAlgorithm, LibItem, Mask, ProcessWrapper, Tasks,
    },
    std::{
        any::TypeId,
        hash::{Hash, Hasher},
        path::PathBuf,
        sync::Arc,
    },
};

#[derive(Debug, Clone)]
pub struct Process {
    size: u32,
    k: usize,
    hamerly: bool,
    calc_unit: CalculationUnit,
    color_space: ColorSpace,
    dist_algo: DistanceAlgorithm,
    filter: Filter,
    img: RgbImage,
    library: Vec<PathBuf>,
    masks: Vec<Mask>,
}

impl Process {
    #[inline(always)]
    pub fn new(
        size: u32,
        k: usize,
        hamerly: bool,
        calc_unit: CalculationUnit,
        color_space: ColorSpace,
        dist_algo: DistanceAlgorithm,
        filter: Filter,
        img: RgbImage,
        library: Vec<PathBuf>,
        masks: Vec<Mask>,
    ) -> Self {
        Self {
            size,
            k,
            hamerly,
            calc_unit,
            color_space,
            dist_algo,
            filter,
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
                    let proc = ProcessWrapper::new(
                        s.size,
                        s.k,
                        s.hamerly,
                        s.calc_unit,
                        s.color_space,
                        s.dist_algo,
                        s.filter.into(),
                    );
                    let lib = Vec::with_capacity(s.library.len());
                    let tasks = proc.index(&s.library);
                    (Progress::None, State::Indexing(s, proc, tasks, lib))
                }),

                State::Indexing(s, proc, mut tasks, mut lib) => Some(match tasks.pop_front() {
                    Some(task) => {
                        if let Some(i) = task.await {
                            lib.push(i);
                        }
                        (Progress::Indexing, State::Indexing(s, proc, tasks, lib))
                    }
                    None => match lib.is_empty() {
                        true => (
                            Progress::Error(String::from("Library is empty.")),
                            State::Finished,
                        ),
                        false => {
                            let (width, height) = s.img.dimensions();
                            let img_buf: RgbImage = ImageBuffer::new(width, height);
                            let tasks = proc.fill(Arc::new(s.img), Arc::new(lib), &s.masks);
                            (Progress::None, State::Filling(proc, tasks, img_buf))
                        }
                    },
                }),

                State::Filling(proc, mut tasks, mut img_buf) => Some(match tasks.pop_front() {
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
        Box<Process>,
        ProcessWrapper,
        Tasks<Option<LibItem>>,
        Vec<LibItem>,
    ),
    Filling(ProcessWrapper, Tasks<(Mask, Arc<RgbImage>)>, RgbImage),
    Finished,
}
