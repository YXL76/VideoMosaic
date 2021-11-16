use {
    crate::states::IMAGE_FILTER,
    async_std::task::JoinHandle,
    iced::{
        futures::stream::{unfold, BoxStream},
        Subscription,
    },
    iced_native::subscription,
    image_crawler::{download_urls, get_urls, Result},
    std::{
        collections::VecDeque,
        hash::{Hash, Hasher},
        path::PathBuf,
    },
};

#[derive(Debug, Clone)]
pub struct Crawler {
    id: usize,
    keyword: String,
    num: usize,
    cnt: usize,
    folder: PathBuf,
}

impl Crawler {
    #[inline(always)]
    pub fn new(id: usize, keyword: String, num: usize, folder: PathBuf) -> Self {
        Self {
            id,
            keyword,
            num,
            cnt: 0,
            folder,
        }
    }

    #[inline(always)]
    pub fn subscription(&self) -> Subscription<Progress> {
        Subscription::from_recipe(self.clone())
    }

    #[inline(always)]
    pub fn add(&mut self) {
        self.cnt += 100;
    }

    #[inline(always)]
    pub fn percentage(&self) -> f32 {
        self.cnt as f32 / self.num as f32
    }

    #[inline(always)]
    pub fn folder(&self) -> &PathBuf {
        &self.folder
    }
}

impl<H, E> subscription::Recipe<H, E> for Crawler
where
    H: Hasher,
{
    type Output = Progress;

    fn hash(&self, state: &mut H) {
        std::any::TypeId::of::<Self>().hash(state);

        self.id.hash(state);
    }

    fn stream(self: Box<Self>, _input: BoxStream<'static, E>) -> BoxStream<'static, Self::Output> {
        let id = self.id;
        let keyword = self.keyword;
        let num = self.num;
        let folder = self.folder;

        Box::pin(unfold(
            State::Ready(keyword, num, folder),
            move |state| async move {
                match state {
                    State::Ready(keyword, num, folder) => {
                        Some(match get_urls(keyword, num).await {
                            Ok((num, tasks)) => {
                                let urls = Vec::with_capacity(num);
                                (Progress::None, State::Getting(tasks, urls, folder))
                            }
                            Err(e) => (Progress::Error(id, e.to_string()), State::Finished),
                        })
                    }
                    State::Getting(mut tasks, mut urls, folder) => Some(match tasks.pop_front() {
                        Some(task) => {
                            if let Ok(ret) = task.await {
                                urls.extend_from_slice(&ret);
                            }
                            (Progress::None, State::Getting(tasks, urls, folder))
                        }
                        None => {
                            let tasks = download_urls(urls, &IMAGE_FILTER, folder.clone());
                            (Progress::None, State::Downloading(tasks, false))
                        }
                    }),
                    State::Downloading(mut tasks, mut flag) => Some(match tasks.pop_front() {
                        Some(task) => {
                            if let Ok(true) = task.await {
                                flag = true;
                            }
                            (Progress::Downloading(id), State::Downloading(tasks, flag))
                        }
                        None => (
                            if flag {
                                Progress::Finished(id)
                            } else {
                                Progress::Error(id, String::from("No images were downloaded"))
                            },
                            State::Finished,
                        ),
                    }),
                    State::Finished => None,
                }
            },
        ))
    }
}

#[derive(Debug, Clone)]
pub enum Progress {
    Downloading(usize),
    Finished(usize),
    None,
    Error(usize, String),
}

#[derive(Debug)]
enum State {
    Ready(String, usize, PathBuf),
    Getting(
        VecDeque<JoinHandle<Result<Vec<String>>>>,
        Vec<String>,
        PathBuf,
    ),
    Downloading(VecDeque<JoinHandle<Result<bool>>>, bool),
    Finished,
}
