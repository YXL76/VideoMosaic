use {
    futures::stream::{FuturesUnordered, StreamExt},
    serde::{Deserialize, Serialize},
    std::{convert::TryInto, fs::File, io::Write, path::PathBuf, time::Duration},
    surf::{Client, Config, Result, Url},
};

const BASE_URL: &str = "https://image.baidu.com/search/acjson";
const PAGE_NUM: usize = 50;
const TIMEOUT: u64 = 60;
const CONCURRENT: usize = 24;
const BASE_PARAMS: [(&str, &str); 8] = [
    ("pn", "0"),
    ("face", "0"),
    ("ie", "utf-8"),
    ("ipn", "rj"),
    ("oe", "utf-8"),
    ("pn", "0"),
    ("rn", "50"),
    ("tn", "resultjson_com"),
];

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

pub async fn download_urls(urls: Vec<String>, filter: &[&str], folder: PathBuf) -> Result<bool> {
    let client: Client = Config::new()
    .set_http_keep_alive(true)
    .set_max_connections_per_host(CONCURRENT)
    .set_timeout(Some(Duration::from_secs(TIMEOUT)))
    .add_header("user-agent", "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/95.0.4638.69 Safari/537.36")?
    .try_into()?;

    let mut reqs = urls
        .iter()
        .enumerate()
        .map(|(idx, url)| download_url(url, &client, filter, &folder, idx))
        .collect::<FuturesUnordered<_>>();

    let mut ans = false;
    while let Some(ret) = reqs.next().await {
        if let Ok(true) = ret {
            ans = true;
        }
    }
    Ok(ans)
}

async fn download_url(
    url: &String,
    client: &Client,
    filter: &[&str],
    folder: &PathBuf,
    idx: usize,
) -> Result<bool> {
    let mut res = client.get(url).await?;
    if let Some(t) = res.content_type() {
        let ext = t.subtype();
        if filter.contains(&ext) {
            let bytes = res.body_bytes().await?;
            let mut file = File::create(folder.join(format!("{}.{}", idx, ext))).unwrap();
            file.write(&bytes).unwrap();
            return Ok(true);
        }
    }
    Ok(false)
}

pub async fn get_urls(keyword: &str, num: usize) -> Result<Vec<String>> {
    let client: Client = Config::new()
        .set_http_keep_alive(true)
        .set_max_connections_per_host(CONCURRENT)
        .set_timeout(Some(Duration::from_secs(TIMEOUT)))
        .add_header("user-agent", "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/95.0.4638.69 Safari/537.36")?
        .try_into()?;

    let mut params = BASE_PARAMS.to_vec();
    params.push(("queryWord", keyword));
    params.push(("word", keyword));
    let url = Url::parse_with_params(BASE_URL, &params).unwrap();

    let mut res = client.get(url).await?;
    let Res { displayNum, .. } = res.body_json().await?;
    let num = num.min(displayNum);

    let mut reqs = (0..num)
        .step_by(PAGE_NUM)
        .map(|start| get_part_urls(start.to_string(), params.clone(), &client))
        .collect::<FuturesUnordered<_>>();

    let mut ans = Vec::with_capacity(num);
    while let Some(ret) = reqs.next().await {
        if let Ok(ret) = ret.as_ref() {
            ans.extend_from_slice(ret);
        }
    }
    Ok(ans)
}

async fn get_part_urls(
    start: String,
    mut params: Vec<(&str, &str)>,
    client: &Client,
) -> Result<Vec<String>> {
    params[0].1 = start.as_str();
    let url = Url::parse_with_params(BASE_URL, &params).unwrap();
    let mut res = client.get(url).await?;
    let Res { data, .. } = res.body_json().await?;

    let mut ret = Vec::with_capacity(data.len());
    for i in data {
        if let Some(str) = i.thumbURL {
            ret.push(str);
        }
    }
    Ok(ret)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    #[test]
    fn get_urls() {
        use {crate::PAGE_NUM, async_std::task::block_on};
        block_on(super::get_urls("风景", PAGE_NUM * 2)).unwrap();
    }

    #[test]
    fn download_urls() {
        use {crate::PAGE_NUM, async_std::task::block_on};
        let urls = block_on(super::get_urls("风景", PAGE_NUM * 2)).unwrap();
        block_on(super::download_urls(
            urls,
            &["png", "jpeg", "jpg"],
            PathBuf::new(),
        ))
        .unwrap();
    }
}
