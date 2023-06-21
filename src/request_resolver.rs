use hyper::{Body, Request, Response, Uri};
use std::error::Error;
use std::fmt::Display;

use tokio::fs;

use crate::backend::BackendTrait;

use crate::manga_list;

#[derive(Debug, Clone)]
struct StringError {
    e: String,
}
impl Display for StringError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error:{}", self.e)
    }
}
impl Error for StringError {}
impl StringError {
    pub fn new(s: &str) -> Self {
        Self { e: s.to_owned() }
    }
}

pub fn err<T>(s: &str) -> anyhow::Result<T> {
    Err(StringError::new(s).into())
}

pub async fn resolve<SelectedBackend: BackendTrait>(
    req: Request<Body>,
) -> anyhow::Result<Response<Body>> {
    let uri = req.uri();
    let path = uri.path().to_owned();
    //dbg!(&path);
    let p: Vec<&str> = path.split('/').collect();
    //dbg!(&p);
    let first_path = match p.get(1) {
        Some(v) => v.to_owned().to_owned(),
        None => return err("why no 1"),
    };

    let response = match &first_path {
        v if v == "favicon.ico" => {
            let f = fs::read("res/favicon.ico").await?;
            Response::new(Body::from(f))
        }
        v if v.is_empty() => {
            let f = fs::read("res/html/index.html").await?;
            Response::new(Body::from(f))
        }
        v if v == "pic" => {
            let name = p.get(2);
            if let Some(name) = name {
                let f = get_res("pic", name).await?;
                Response::new(Body::from(f))
            } else {
                return err("img not found");
            }
        }

        v if v == "css" => {
            let name = p.get(2);
            if let Some(name) = name {
                let f = get_res("css", name).await?;
                Response::new(Body::from(f))
            } else {
                return err("img not found");
            }
        }

        v if v == "reader" => Response::new(Body::from(fs::read(r"res/html/reader.html").await?)),

        v if v == "manga_page" => {
            Response::new(Body::from(fs::read(r"res/html/manga.html").await?))
        }

        v if v == "info" => {
            let info = p.get(2);
            if let Some(info) = info {
                let i = get_info(info, uri).await?;
                Response::new(Body::from(i))
            } else {
                return err("img not found");
            }
        }

        v if v == "manga" => {
            let manga_id = match p.get(2) {
                Some(v) => v,
                None => return err("manga id needed"),
            }
            .to_string();

            let chapter = match p.get(3) {
                Some(v) => v,
                None => {
                    let out = {
                        serde_json::to_string(
                            match { manga_list::get_list_ref().get_list_mut().get(&manga_id) } {
                                Some(v) => v,
                                None => return err("manga not found"),
                            },
                        )?
                    };
                    return Ok(Response::new(Body::from(out)));
                }
            }
            .to_string();

            let base_path = (*manga_list::get_list_ref().path).to_owned();

            let pic_id = match p.get(4) {
                Some(v) => v,
                None => {
                    let info =
                        SelectedBackend::get_chapter_info(&base_path, &manga_id, &chapter).await?;
                    let out = serde_json::to_string(&info).unwrap();
                    return Ok(Response::new(Body::from(out)));
                }
            }
            .parse::<usize>()?;

            Response::new(Body::from(
                SelectedBackend::get_pic_in_chapter(&base_path, &manga_id, &chapter, pic_id)
                    .await?
                    .unwrap(),
            ))
        }

        _ => {
            let f = fs::read("res/html/404.html").await?;

            Response::new(Body::from(f))
        }
    };

    Ok(response)
}

async fn get_res(t: &str, name: &str) -> Result<Vec<u8>, std::io::Error> {
    let f = fs::read(&format!("res/{}/{}", t, name)).await?;
    Ok(f)
}

async fn get_info(info: &str, _uri: &Uri) -> Result<Vec<u8>, std::io::Error> {
    let out = match info {
        i if i == "all_manga" => manga_list::get_list_ref().all_json().into_bytes(),
        _ => {
            return Ok("".as_bytes().to_owned());
        }
    };

    Ok(out)
}
