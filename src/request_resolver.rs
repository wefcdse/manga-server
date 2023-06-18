use hyper::{Body, Request, Response, Uri};
use std::error::Error;
use std::fmt::Display;

use tokio::fs;

use crate::{
    dmzj,
    manga_list::{self, MangaInfo},
};

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

pub async fn resolve(req: Request<Body>) -> anyhow::Result<Response<Body>> {
    let uri = req.uri();
    let path = uri.path().to_owned();
    //dbg!(&path);
    let p: Vec<&str> = path.split("/").collect();
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
        v if v == "" => {
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
                let i = get_info(*info, &uri).await?;
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
            .parse::<u64>()?
            .to_string();
            let hua = match p.get(3) {
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
            .parse::<u64>()?
            .to_string();
            let base_path = (&*manga_list::get_list_ref().path).to_owned();

            let pic_id = match p.get(4) {
                Some(v) => v,
                None => {
                    #[derive(Debug, serde::Serialize)]
                    struct Info {
                        length: usize,
                    };
                    let l = dmzj::get_zip_length(&base_path, &manga_id, &hua).await?;
                    let i = Info { length: l };
                    let out = serde_json::to_string(&i)?;
                    return Ok(Response::new(Body::from(out)));
                }
            }
            .parse::<usize>()?;

            Response::new(Body::from(
                dmzj::get_pic_in_zip(&base_path, &manga_id, &hua, pic_id)
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
    let f = fs::read((&format!("res/{}/{}", t, name))).await?;
    Ok(f)
}

async fn get_info(info: &str, uri: &Uri) -> Result<Vec<u8>, std::io::Error> {
    let out = match info {
        i if i == "all_manga" => manga_list::get_list_ref().all_json().into_bytes(),
        _ => {
            return Ok("".as_bytes().to_owned());
        }
    };

    Ok(out)
}
