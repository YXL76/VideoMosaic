use {
    iced::{
        futures::stream::{unfold, BoxStream},
        Subscription,
    },
    iced_native::subscription,
    mosaic_video_crawler::{download_urls, gen_client, get_urls, HttpClient, Result, TasksIter},
    std::{
        any::TypeId,
        hash::{Hash, Hasher},
        path::PathBuf,
        sync::Arc,
    },
};

#[derive(Debug, Clone)]
pub struct Crawler {
    id: usize,
    keyword: String,
    num: usize,
    step: f32,
    percentage: f32,
    folder: PathBuf,
}

impl Crawler {
    #[inline(always)]
    pub fn new(id: usize, keyword: String, num: usize, folder: PathBuf) -> Self {
        Self {
            id,
            keyword,
            num,
            step: 100. / num as f32,
            percentage: 0.,
            folder,
        }
    }

    #[inline(always)]
    pub fn subscription(&self) -> Subscription<Progress> {
        Subscription::from_recipe(self.clone())
    }

    #[inline(always)]
    pub fn add(&mut self) {
        self.percentage += self.step;
    }

    #[inline(always)]
    pub fn percentage(&self) -> f32 {
        self.percentage
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
        TypeId::of::<Self>().hash(state);
        self.id.hash(state);
    }

    fn stream(self: Box<Self>, _input: BoxStream<'static, E>) -> BoxStream<'static, Self::Output> {
        let id = self.id;
        let keyword = self.keyword;
        let num = self.num;
        let folder = self.folder;

        Box::pin(unfold(
            State::Ready(gen_client(), keyword, num, folder),
            move |state| async move {
                match state {
                    State::Ready(client, keyword, num, folder) => {
                        Some(match get_urls(client.clone(), keyword, num).await {
                            Ok((num, tasks)) => {
                                let tasks = tasks.into_iter();
                                let urls = Vec::with_capacity(num);
                                (Progress::None, State::Getting(client, tasks, urls, folder))
                            }
                            Err(e) => (Progress::Error(id, e.to_string()), State::Finished),
                        })
                    }

                    State::Getting(client, mut tasks, mut urls, folder) => {
                        Some(match tasks.next() {
                            Some(task) => {
                                if let Ok(ret) = task.await {
                                    urls.extend_from_slice(&ret);
                                }
                                (Progress::None, State::Getting(client, tasks, urls, folder))
                            }
                            None => {
                                let tasks = download_urls(client, urls, folder.clone()).into_iter();
                                (Progress::None, State::Downloading(tasks, false))
                            }
                        })
                    }

                    State::Downloading(mut tasks, mut flag) => Some(match tasks.next() {
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
                                Progress::Error(id, String::from("No images were downloaded."))
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
    Ready(Arc<HttpClient>, String, usize, PathBuf),
    Getting(
        Arc<HttpClient>,
        TasksIter<Result<Vec<String>>>,
        Vec<String>,
        PathBuf,
    ),
    Downloading(TasksIter<Result<bool>>, bool),
    Finished,
}
