use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::{
    backend::{BackendTrait, ChapterInfo},
    manga_list::{ChapterBasicInfo, MangaInfo, MangaList},
    utils::ToResult,
};
use anyhow::Result;
#[derive(Debug, Clone, Copy)]
pub struct Shaft;

lazy_static::lazy_static! {
    static ref INFO: HashMap<String, MangaInfoLocal> = singel_info();
}

#[test]
fn t_impl() {
    // Shaft::generate_manga_list();
}
#[async_trait::async_trait]
impl BackendTrait for Shaft {
    fn generate_manga_list() -> MangaList {
        #[derive(Debug, Default, serde::Deserialize)]
        struct Config {
            path: String,
        }
        let f: String = std::fs::read_to_string("shaft.toml").unwrap();
        let config: Config = toml::from_str(&f).unwrap();
        let infos = read_all_info(&config.path);
        let out_map: HashMap<String, MangaInfo> = infos
            .into_iter()
            .map(|(_k, v)| {
                (
                    format!("{}", v.id),
                    MangaInfo {
                        name: v.name.clone(),
                        pic: format!("/manga/{}/single/0", v.id),
                        id: format!("{}", v.id),
                        chapters: vec![ChapterBasicInfo {
                            length: v.all_pages,
                            name: v.name,
                            id: "single".to_string(),
                        }],
                    },
                )
            })
            .collect();

        // dbg!(infos);
        MangaList {
            list: Arc::new(Mutex::new(out_map)),
            path: Arc::new(config.path),
            all_basic_info: Arc::new(Mutex::new(None)),
        }
    }
    async fn get_pic_in_chapter(
        _base_path: &str,
        manga_id: &str,
        _chapter: &str,
        pic_id: usize,
    ) -> Result<Option<Vec<u8>>> {
        let info = INFO.get(manga_id).to_result()?;
        let path = if info.is_single {
            &info.full_paths.get(0).to_result()?.1
        } else {
            &info
                .full_paths
                .iter()
                .filter(|(k, _v)| *k == pic_id + 1)
                .next()
                .to_result()?
                .1
        };
        // dbg!(path);
        let out = tokio::fs::read(path).await?;
        Ok(Some(out))
    }
    async fn get_chapter_info(
        _base_path: &str,
        manga_id: &str,
        _chapter: &str,
    ) -> Result<ChapterInfo> {
        let info = INFO.get(manga_id).to_result()?;
        Ok(ChapterInfo {
            length: info.all_pages,
            name: info.name.clone(),
        })
    }
}

#[test]
fn t_str() {
    let path = r"F:\media\ShaftImages";
    read_all_info(path);
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct MangaInfoLocal {
    id: usize,
    name: String,
    all_pages: usize,
    is_single: bool,
    extend_name: String,
    full_paths: Vec<(usize, String)>,
}

fn read_all_info(path: &str) -> HashMap<usize, MangaInfoLocal> {
    use crate::utils::ShortUnwrap;
    let mut map: HashMap<usize, MangaInfoLocal> = HashMap::new();
    let all_extend_name = vec!["jpg", "png", "gif"];
    for e in walkdir::WalkDir::new(path) {
        let e = e.unwrap();
        if !e.file_type().is_file() {
            continue;
        }
        let full_name = e.file_name().to_str().unwrap();
        let extend_name = full_name.split('.').last().unwrap();
        if !all_extend_name.contains(&extend_name) {
            // dbg!(extend_name);
            continue;
        }
        let name = full_name[..(full_name.len() - 4)].to_owned();
        if name.ends_with(')') {
            // dbg!(name);
            continue;
        }
        // dbg!(&name);
        let splited = name.split('_').collect::<Vec<_>>();
        if !splited.last().u().starts_with('p') {
            // dbg!(&name);
            let id: usize = splited[splited.len() - 1].parse().u();
            let pic_name: String = name[..name.len() - (splited.last().u().len() + 1)].to_owned();
            // dbg!(id);
            // dbg!(&pic_name);
            match map.get_mut(&id) {
                Some(v) => {
                    v.all_pages += 1;
                }
                None => {
                    map.insert(
                        id,
                        MangaInfoLocal {
                            id,
                            name: format!("s-{}", pic_name),
                            all_pages: 1,
                            extend_name: extend_name.to_owned(),
                            is_single: true,
                            full_paths: vec![(0, e.path().as_os_str().to_str().u().to_owned())],
                        },
                    );
                }
            }
        } else {
            // dbg!(&name);
            let split_len = splited.len();
            let id: usize = splited[splited.len() - 2].parse().u();
            let pic_name: String = name
                [..name.len() - (splited[split_len - 1].len() + splited[split_len - 2].len() + 2)]
                .to_owned();

            let page: usize = splited[split_len - 1][1..].parse().u();
            // dbg!(page);
            // dbg!(id);
            // dbg!(&pic_name);
            match map.get_mut(&id) {
                Some(v) => {
                    v.all_pages += 1;
                    v.full_paths
                        .push((page, e.path().as_os_str().to_str().u().to_owned()));
                }
                None => {
                    map.insert(
                        id,
                        MangaInfoLocal {
                            id,
                            name: format!("m-{}", pic_name),
                            all_pages: 1,
                            extend_name: extend_name.to_owned(),
                            is_single: false,
                            full_paths: vec![(page, e.path().as_os_str().to_str().u().to_owned())],
                        },
                    );
                }
            }
        }
    }
    // dbg!(map);
    // for (k, v) in map {
    //     dbg!(v);
    // }
    map
}

fn singel_info() -> HashMap<String, MangaInfoLocal> {
    #[derive(Debug, Default, serde::Deserialize)]
    struct Config {
        path: String,
    }
    let f: String = std::fs::read_to_string("shaft.toml").unwrap();
    let config: Config = toml::from_str(&f).unwrap();
    let infos = read_all_info(&config.path);
    infos
        .into_iter()
        .map(|(k, v)| (format!("{}", k), v))
        .collect()
}
