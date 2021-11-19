use {
    anyhow::Result,
    async_std::{
        fs::File,
        future::timeout,
        io::WriteExt,
        task::{spawn, JoinHandle},
    },
    http::{Method, Uri},
    isahc::{config::VersionNegotiation, prelude::*, Request},
    mime::Mime,
    serde::Deserialize,
    std::{borrow::Cow, path::PathBuf, sync::Arc, time::Duration},
    urlencoding::encode,
};

pub use isahc::HttpClient;

type PARAMS = [(&'static str, Cow<'static, str>); 10];

const BASE_URL: &str = "https://image.baidu.com/search/acjson";
const PAGE_NUM: usize = 50;
const TIMEOUT: Duration = Duration::from_secs(60);
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
Chrome/96.0.4664.45 Safari/537.36";
const HEADERS: [(&'static str, &'static str); 6] = [
    ("Accept", "image/jpeg, image/png, */*; q=0.9"),
    ("Accept-Encoding", "gzip, deflate, br"),
    ("Accept-Language", "zh-CN,zh;q=0.9"),
    ("Host", "image.baidu.com"),
    ("Referer", "https://image.baidu.com/"),
    ("User-Agent", USER_AGENT),
];

#[allow(non_snake_case)]
#[derive(Debug, Deserialize)]
struct ImgData {
    thumbURL: Option<String>,
}

#[allow(non_snake_case)]
#[derive(Debug, Deserialize)]
struct Res {
    displayNum: usize,
    data: Vec<ImgData>,
}

pub fn download_urls(
    client: Arc<HttpClient>,
    urls: Vec<String>,
    filter: &'static [&str],
    folder: PathBuf,
) -> Vec<JoinHandle<Result<bool>>> {
    urls.into_iter()
        .enumerate()
        .map(|(idx, url)| {
            let url = url.clone();
            let client = client.clone();
            spawn(download_url(client, url, filter, folder.clone(), idx))
        })
        .collect::<Vec<_>>()
}

async fn download_url(
    client: Arc<HttpClient>,
    url: String,
    filter: &[&str],
    folder: PathBuf,
    idx: usize,
) -> Result<bool> {
    let url = url.parse::<Uri>().unwrap();
    let request = Request::builder()
        .method(Method::GET)
        .header("Host", url.host().unwrap())
        .uri(url)
        .version_negotiation(VersionNegotiation::http2())
        .body(())
        .unwrap();

    let mut res = client.send_async(request).await?;
    if let Some(typ) = res.headers().get("content-type") {
        let typ = typ.to_str().unwrap().parse::<Mime>().unwrap();
        let ext = typ.subtype().as_str();
        if filter.contains(&ext) {
            let bytes = timeout(TIMEOUT, res.bytes()).await??;
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
    client: Arc<HttpClient>,
    keyword: String,
    num: usize,
) -> Result<(usize, Vec<JoinHandle<Result<Vec<String>>>>)> {
    let keyword = encode(&keyword).to_string();
    let mut params = BASE_PARAMS.clone();
    params[0].1 = keyword.clone().into();
    params[1].1 = keyword.into();
    let url = parse_url(&params);

    let mut res = client.get_async(url).await?;
    let Res { displayNum, .. } = res.json::<Res>().await?;
    let num = num.min(displayNum);

    let tasks = (0..num)
        .step_by(PAGE_NUM)
        .map(|start| {
            let client = client.clone();
            spawn(get_part_urls(client, start.to_string(), params.clone()))
        })
        .collect::<Vec<_>>();

    Ok((num, tasks))
}

async fn get_part_urls(
    client: Arc<HttpClient>,
    start: String,
    mut params: PARAMS,
) -> Result<Vec<String>> {
    params[2].1 = start.into();
    let url = parse_url(&params);
    let mut res = client.get_async(url).await?;
    let Res { data, .. } = res.json::<Res>().await?;

    let mut ret = Vec::with_capacity(data.len());
    for ImgData { thumbURL } in data {
        if let Some(str) = thumbURL {
            ret.push(str);
        }
    }
    Ok(ret)
}

fn parse_url(params: &PARAMS) -> Uri {
    format!(
        "{}?{}",
        BASE_URL,
        params
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join("&")
    )
    .parse::<Uri>()
    .unwrap()
}

pub fn gen_client() -> HttpClient {
    HttpClient::builder()
        .timeout(TIMEOUT)
        .max_connections(CONCURRENT)
        .default_headers(&HEADERS)
        .build()
        .unwrap()
}

#[cfg(test)]
mod tests {
    use {
        super::PAGE_NUM,
        async_std::task::block_on,
        isahc::HttpClient,
        std::{
            fs::{create_dir, remove_dir_all},
            path::PathBuf,
            sync::Arc,
        },
    };

    fn get_urls(client: Arc<HttpClient>) -> Vec<String> {
        block_on(async {
            let (num, tasks) = super::get_urls(client, "风景".into(), PAGE_NUM * 2)
                .await
                .unwrap();
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

        let client = Arc::new(super::gen_client());
        let urls = get_urls(client.clone());
        block_on(async {
            let folder = PathBuf::from("test");
            let _ = remove_dir_all(&folder);
            create_dir(&folder).unwrap();
            let tasks = super::download_urls(client, urls, &FILTER, folder);
            for task in tasks {
                let _ = task.await;
            }
        });
    }
}
