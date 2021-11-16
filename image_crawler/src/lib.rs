use {
    async_std::{
        fs::File,
        io::WriteExt,
        task::{spawn, JoinHandle},
    },
    serde::{Deserialize, Serialize},
    std::{borrow::Cow, collections::VecDeque, path::PathBuf, sync::Arc, time::Duration},
    surf::{Client, Config, Url},
};

pub use surf::Result;

type PARAMS = [(&'static str, Cow<'static, str>); 10];

const BASE_URL: &str = "https://image.baidu.com/search/acjson";
const PAGE_NUM: usize = 50;
const TIMEOUT: u64 = 60;
const CONCURRENT: usize = 24;
const BASE_PARAMS: PARAMS = [
    ("queryWord", Cow::Borrowed("")),
    ("word", Cow::Borrowed("")),
    ("pn", Cow::Borrowed("0")),
    ("face", Cow::Borrowed("0")),
    ("ie", Cow::Borrowed("utf-8")),
    ("ipn", Cow::Borrowed("rj")),
    ("oe", Cow::Borrowed("utf-8")),
    ("pn", Cow::Borrowed("0")),
    ("rn", Cow::Borrowed("50")),
    ("tn", Cow::Borrowed("resultjson_com")),
];
const USER_AGENT: &str = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) \
Chrome/95.0.4638.69 Safari/537.36";

#[allow(non_snake_case)]
#[derive(Debug, Deserialize, Serialize)]
struct ImgData {
    thumbURL: Option<String>,
}

#[allow(non_snake_case)]
#[derive(Debug, Deserialize, Serialize)]
struct Res {
    displayNum: usize,
    data: Vec<ImgData>,
}

pub fn download_urls(
    urls: Vec<String>,
    filter: &'static [&str],
    folder: PathBuf,
) -> VecDeque<JoinHandle<Result<bool>>> {
    let client = Arc::new(gen_client());

    urls.iter()
        .enumerate()
        .map(|(idx, url)| {
            let url = url.clone();
            let client = client.clone();
            spawn(download_url(url, client, filter, folder.clone(), idx))
        })
        .collect::<VecDeque<_>>()
}

async fn download_url(
    url: String,
    client: Arc<Client>,
    filter: &[&str],
    folder: PathBuf,
    idx: usize,
) -> Result<bool> {
    let mut res = client.get(url).await?;
    if let Some(t) = res.content_type() {
        let ext = t.subtype();
        if filter.contains(&ext) {
            let bytes = res.body_bytes().await?;
            let mut file = File::create(folder.join(format!("{}.{}", idx, ext)))
                .await
                .unwrap();
            file.write(&bytes).await.unwrap();
            return Ok(true);
        }
    }
    Ok(false)
}

pub async fn get_urls(
    keyword: String,
    num: usize,
) -> Result<(usize, VecDeque<JoinHandle<Result<Vec<String>>>>)> {
    let client = Arc::new(gen_client());

    let mut params = BASE_PARAMS.clone();
    params[0].1 = keyword.clone().into();
    params[1].1 = keyword.into();
    let url = Url::parse_with_params(BASE_URL, &params).unwrap();

    let mut res = client.get(url).await?;
    let Res { displayNum, .. } = res.body_json().await?;
    let num = num.min(displayNum);

    let tasks = (0..num)
        .step_by(PAGE_NUM)
        .map(|start| {
            let client = client.clone();
            spawn(get_part_urls(start.to_string(), params.clone(), client))
        })
        .collect::<VecDeque<_>>();

    Ok((num, tasks))
}

async fn get_part_urls(
    start: String,
    mut params: PARAMS,
    client: Arc<Client>,
) -> Result<Vec<String>> {
    params[2].1 = start.into();
    let url = Url::parse_with_params(BASE_URL, &params).unwrap();
    let mut res = client.get(url).await?;
    let Res { data, .. } = res.body_json().await?;

    let mut ret = Vec::with_capacity(data.len());
    for ImgData { thumbURL } in data {
        if let Some(str) = thumbURL {
            ret.push(str);
        }
    }
    Ok(ret)
}

fn gen_client() -> Client {
    Config::new()
        .set_http_keep_alive(true)
        .set_max_connections_per_host(CONCURRENT)
        .set_timeout(Some(Duration::from_secs(TIMEOUT)))
        .add_header("user-agent", USER_AGENT)
        .unwrap()
        .try_into()
        .unwrap()
}

#[cfg(test)]
mod tests {
    use {
        super::PAGE_NUM,
        async_std::task::block_on,
        std::{
            fs::{create_dir, remove_dir_all},
            path::PathBuf,
        },
    };

    fn get_urls() -> Vec<String> {
        block_on(async {
            let (num, tasks) = super::get_urls("风景".into(), PAGE_NUM * 2).await.unwrap();
            let mut urls = Vec::with_capacity(num);
            for task in tasks {
                if let Ok(ret) = task.await {
                    urls.extend_from_slice(&ret);
                }
            }
            urls
        })
    }

    #[test]
    fn download_urls() {
        const FILTER: [&str; 3] = ["png", "jpeg", "jpg"];

        let urls = get_urls();
        block_on(async {
            let folder = PathBuf::from("test");
            let _ = remove_dir_all(&folder);
            create_dir(&folder).unwrap();
            let tasks = super::download_urls(urls, &FILTER, folder);
            for task in tasks {
                let _ = task.await;
            }
        });
    }
}
