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

pub fn crawler(keyword: String, num: usize, folder: PathBuf) -> Subscription<Progress> {
    Subscription::from_recipe(Crawler {
        keyword,
        num,
        folder,
    })
}

pub struct Crawler {
    keyword: String,
    num: usize,
    folder: PathBuf,
}

impl<H, E> subscription::Recipe<H, E> for Crawler
where
    H: Hasher,
{
    type Output = Progress;

    fn hash(&self, state: &mut H) {
        std::any::TypeId::of::<Self>().hash(state);

        self.folder.hash(state);
    }

    fn stream(self: Box<Self>, _input: BoxStream<'static, E>) -> BoxStream<'static, Self::Output> {
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
                                (Progress::Started, State::Getting(tasks, urls, folder))
                            }
                            Err(_) => (Progress::Error, State::Finished),
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
                            let tasks = download_urls(urls, &IMAGE_FILTER, folder);
                            (Progress::None, State::Downloading(tasks, false))
                        }
                    }),
                    State::Downloading(mut tasks, mut flag) => Some(match tasks.pop_front() {
                        Some(task) => {
                            if let Ok(true) = task.await {
                                flag = true;
                            }
                            (Progress::Downloading, State::Downloading(tasks, flag))
                        }
                        None => (Progress::Finished, State::Finished),
                    }),
                    State::Finished => None,
                }
            },
        ))
    }
}

#[derive(Copy, Clone)]
pub enum Progress {
    Started,
    Downloading,
    Finished,
    None,
    Error,
}

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
