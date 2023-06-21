use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use anyhow::Ok;
use serde::Serialize;

use crate::{
    backend::{BackendTrait, ChapterInfo},
    manga_list::{self, ChapterBasicInfo, MangaInfo, MangaList},
    utils::ToResult,
};

#[derive(Debug, Clone, Copy)]
pub struct Eh;

lazy_static::lazy_static! {
    static ref INFO: HashMap<String, MangaInfoLocal> = singel_info();
}

fn singel_info() -> HashMap<String, MangaInfoLocal> {
    #[derive(Debug, Default, serde::Deserialize)]
    struct Config {
        path: String,
    }
    let f = std::fs::read_to_string("eh.toml").unwrap();
    let config: Config = toml::from_str(&f).unwrap();

    read_all_infos(&config.path)
}

#[test]
fn t_impl_backend() {
    dbg!(Eh::generate_manga_list());
}
#[async_trait::async_trait]
impl BackendTrait for Eh {
    fn generate_manga_list() -> MangaList {
        #[derive(Debug, Default, serde::Deserialize)]
        struct Config {
            path: String,
        }
        let f: String = std::fs::read_to_string("eh.toml").unwrap();
        let config: Config = toml::from_str(&f).unwrap();

        let info = read_all_infos(&config.path);

        let info = info
            .into_iter()
            .map(|(k, v)| {
                let manga_name = k;
                let manga_id = format!("{:?}", md5::compute(&manga_name));
                let o = MangaInfo {
                    name: manga_name,
                    pic: format!("/manga/{}/single/0", manga_id),
                    id: manga_id.clone(),
                    chapters: vec![ChapterBasicInfo {
                        id: "single".to_string(),
                        name: "single".to_string(),
                        length: v.pictures.len(),
                    }],
                };
                (manga_id, o)
            })
            .collect::<HashMap<_, _>>();

        MangaList {
            list: Arc::new(Mutex::new(info)),
            path: Arc::new(config.path),
            all_basic_info: Arc::new(Mutex::new(None)),
        }
    }
    async fn get_pic_in_chapter(
        base_path: &str,
        manga_id: &str,
        _chapter: &str,
        pic_id: usize,
    ) -> anyhow::Result<Option<Vec<u8>>> {
        let manga_name = {
            let list = manga_list::get_list_ref().get_list_mut();
            let info = list.get(manga_id).to_result()?;
            info.name.to_owned()
        };
        let pic_name = INFO
            .get(&manga_name)
            .to_result()?
            .pictures
            .get(pic_id)
            .to_result()?
            .to_owned();
        let path = format!("{}/{}/{}", base_path, manga_name, pic_name);
        // dbg!(&path);
        let out = tokio::fs::read(path).await?;

        Ok(Some(out))
    }
    async fn get_chapter_info(
        _base_path: &str,
        manga_id: &str,
        _chapter: &str,
    ) -> anyhow::Result<ChapterInfo> {
        let l = {
            let list = manga_list::get_list_ref().get_list_mut();
            let info = list.get(manga_id).to_result()?;
            info.chapters.get(0).to_result()?.length
        };
        let out = ChapterInfo {
            length: l,
            name: "single".to_string(),
        };
        Ok(out)
    }
}

#[test]
fn t_read_all_infos() {
    dbg!(read_all_infos(r"H:\g\Books\manga\eh"));
}
#[derive(Debug, Serialize, PartialEq, Clone, Eq)]
struct MangaInfoLocal {
    pictures: Vec<String>,
}

fn read_all_infos(path: &str) -> HashMap<String, MangaInfoLocal> {
    let mut out: HashMap<String, MangaInfoLocal> = HashMap::new();
    let pic_extand_names = vec!["png", "jpg"];
    for e in walkdir::WalkDir::new(path) {
        let e = e.unwrap();
        if e.depth() != 2 {
            continue;
        }
        let picture_name = e.file_name().to_str().unwrap().to_owned();
        let path = e
            .path()
            .to_str()
            .unwrap()
            .chars()
            .map(|c| if c == '\\' { '/' } else { c })
            .collect::<String>();
        let path_vec = path.split('/').collect::<Vec<_>>();
        let manga_name = path_vec[path_vec.len() - 2];

        let n: Vec<&str> = picture_name.split('.').collect();
        let extand_name = n[n.len() - 1];
        if !pic_extand_names.contains(&extand_name) {
            println!("{:?}", e);
            continue;
        };

        match out.get_mut(manga_name) {
            None => {
                let info = MangaInfoLocal {
                    pictures: {
                        let h: Vec<String> = vec![picture_name.to_owned()];
                        h
                    },
                };
                out.insert(manga_name.to_owned(), info);
            }
            Some(v) => {
                if v.pictures.contains(&picture_name) {
                } else {
                    v.pictures.push(picture_name);
                }
            }
        }
    }
    for (_k, v) in out.iter_mut() {
        v.pictures.sort()
    }
    out
}

#[test]
fn test_eh() {
    let path = r"H:\g\Books\manga\eh";
    let w = walkdir::WalkDir::new(path);
    let mut png = 0;
    let mut jpg = 0;
    for e in w {
        let e = e.unwrap();
        let t: std::time::SystemTime = dbg!(e.metadata().unwrap().modified().unwrap());
        println!("{:?}", t.elapsed());
        match e.depth() {
            2 => {
                let name = e.file_name().to_str().unwrap();
                let n: Vec<&str> = name.split('.').collect();
                match n[n.len() - 1] {
                    v if v != "jpg" && v != "png" => {
                        println!("{:?}", e);
                    }
                    v if v == "png" => {
                        png += 1;
                    }
                    v if v == "jpg" => {
                        jpg += 1;
                    }
                    _ => {
                        println!("{:?}", e);
                        //panic!()
                    }
                }
            }
            _ => {}
        }
    }
    println!("png:{},jpg:{}", png, jpg);
}
