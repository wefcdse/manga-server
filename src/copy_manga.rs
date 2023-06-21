#[derive(Debug, Clone, Copy)]
pub struct CopyManga;

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use serde::Serialize;

use crate::{
    backend::{BackendTrait, ChapterInfo},
    manga_list::{self, ChapterBasicInfo, MangaInfo, MangaList},
    utils::ToResult,
};

#[test]
fn t() {
    let hasher: md5::Digest = md5::compute("0");
    let a = format!("{:?}", hasher);
    dbg!(a);
}
#[test]
fn t_impl_backend() {
    dbg!(CopyManga::generate_manga_list());
}

#[async_trait::async_trait]
impl BackendTrait for CopyManga {
    fn generate_manga_list() -> MangaList {
        #[derive(Debug, Default, serde::Deserialize)]
        struct Config {
            path: String,
        }
        let f = std::fs::read_to_string("copy_manga.toml").unwrap();
        let config: Config = toml::from_str(&f).unwrap();
        let infos = read_all_infos(&config.path);
        let mut list = infos
            .iter()
            .map(|(k, v)| {
                let md5 = format!("{:?}", md5::compute(k));
                let out = MangaInfo {
                    name: k.clone(),
                    pic: String::new(),
                    id: md5.clone(),
                    chapters: {
                        let mut c: Vec<&String> = v.chapters.keys().collect::<Vec<_>>();
                        c.sort();
                        let mut out = Vec::new();
                        for chapter in c {
                            let md5 = format!("{:?}", md5::compute(chapter));
                            let info = v.chapters.get(chapter).unwrap();
                            let info = ChapterBasicInfo {
                                id: md5,
                                name: chapter.to_owned(),
                                length: info.length,
                            };
                            out.push(info)
                        }
                        out
                    },
                };
                (md5, out)
            })
            .collect::<HashMap<_, _>>();
        for (k, v) in list.iter_mut() {
            v.pic = format!("/manga/{}/{}/0", k, v.chapters[0].id);
        }

        MangaList {
            list: Arc::new(Mutex::new(list)),
            path: Arc::new(config.path),
            all_basic_info: Arc::new(Mutex::new(None)),
        }
    }
    async fn get_pic_in_chapter(
        base_path: &str,
        manga_id: &str,
        chapter: &str,
        pic_id: usize,
    ) -> anyhow::Result<Option<Vec<u8>>> {
        // dbg!(&(manga_id, chapter, pic_id, bass_path));
        let (manga_name, chapter_name) = {
            let d = manga_list::get_list_ref().get_list_mut();
            let info = d.get(manga_id).to_result()?;
            // dbg!(info);
            let mn = info.name.clone();
            let cn = info
                .chapters
                .iter()
                .filter(|i| i.id == chapter)
                .collect::<Vec<_>>()
                .get(0)
                .to_result()?
                .name
                .clone();
            (mn, cn)
        };
        // dbg!(manga_name, chapter_name);
        let path = format!(
            "{}/{}/{}/{:03}.jpg",
            base_path,
            manga_name,
            chapter_name,
            pic_id + 1
        );
        // dbg!(path);
        let out = tokio::fs::read(path).await?;
        Ok(Some(out))
    }
    async fn get_chapter_info(
        _base_path: &str,
        manga_id: &str,
        chapter: &str,
    ) -> anyhow::Result<ChapterInfo> {
        let (chapter_name, length) = {
            let d = manga_list::get_list_ref().get_list_mut();
            let info = d.get(manga_id).to_result()?;
            let c = info
                .chapters
                .iter()
                .filter(|i| i.id == chapter)
                .collect::<Vec<_>>();
            let ci = c.get(0).to_result()?;
            (ci.name.clone(), ci.length)
        };
        Ok(ChapterInfo {
            length,
            name: chapter_name,
        })
    }
}

#[test]
fn t_read_all_infos() {
    dbg!(read_all_infos(r"d:/aaa"));
}

#[derive(Debug, serde::Serialize, PartialEq, Clone, Eq)]
struct MangaInfoLocal {
    chapters: HashMap<String, ChapterBasicInfoLocal>,
}
#[derive(Debug, Serialize, PartialEq, Clone, Eq)]
pub struct ChapterBasicInfoLocal {
    pub length: usize,
}
fn read_all_infos(path: &str) -> HashMap<String, MangaInfoLocal> {
    let mut h: HashMap<String, MangaInfoLocal> = HashMap::new();
    for e in walkdir::WalkDir::new(path) {
        let e = e.unwrap();
        match e.depth() {
            3 => {
                let p = e.path().as_os_str().to_str().unwrap();
                let p = p
                    .chars()
                    .map(|c| if c == '\\' { '/' } else { c })
                    .collect::<String>();
                let n = p.split('/').collect::<Vec<_>>();
                let manga = n[n.len() - 3];
                let chapter = n[n.len() - 2];
                match h.get_mut(manga) {
                    None => {
                        h.insert(
                            manga.to_string(),
                            MangaInfoLocal {
                                chapters: {
                                    let mut c = HashMap::new();
                                    c.insert(
                                        chapter.to_string(),
                                        ChapterBasicInfoLocal { length: 1 },
                                    );
                                    c
                                },
                            },
                        );
                    }

                    Some(v) => {
                        match v.chapters.get_mut(chapter) {
                            None => {
                                v.chapters.insert(
                                    chapter.to_string(),
                                    ChapterBasicInfoLocal { length: 1 },
                                );
                            }
                            Some(v) => v.length += 1,
                        };
                    }
                };
            }

            _ => continue,
        }
    }
    h
}
