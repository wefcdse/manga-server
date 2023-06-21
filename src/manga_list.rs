use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use serde::Serialize;

use crate::{backend::BackendTrait, copy_manga, dmzj, eh};

// use super::SelectedBackend;

#[derive(Debug, Serialize, PartialEq, Clone, Eq)]
pub struct MangaInfo {
    pub name: String,
    pub pic: String,
    pub id: String,
    pub chapters: Vec<ChapterBasicInfo>,
}
#[derive(Debug, Serialize, PartialEq, Clone, Eq)]
pub struct ChapterBasicInfo {
    pub id: String,
    pub name: String,
    pub length: usize,
}

#[derive(Debug, Serialize, PartialEq, Clone, Eq)]
pub struct MangaBasicInfo {
    pub name: String,
    pub pic: String,
    pub id: String,
    pub first: String,
}

#[derive(Debug, Clone)]
pub struct MangaList {
    pub list: Arc<Mutex<HashMap<String, MangaInfo>>>,
    pub path: Arc<String>,
    pub all_basic_info: Arc<Mutex<Option<Vec<MangaBasicInfo>>>>,
}
impl MangaList {
    pub fn new(path: &str) -> Self {
        Self {
            list: Arc::new(Mutex::new(HashMap::new())),
            path: Arc::new(path.to_owned()),
            all_basic_info: Arc::new(Mutex::new(None)),
        }
    }
    pub fn collect_info(&self) {
        if self.all_basic_info.lock().unwrap().is_none() {
            *self.all_basic_info.lock().unwrap() = Some(
                self.list
                    .lock()
                    .unwrap()
                    .iter()
                    .map(|(_, v)| MangaBasicInfo {
                        name: v.name.to_owned(),
                        pic: v.pic.to_owned(),
                        id: v.id.to_owned(),
                        first: if let Some(e) = v.chapters.get(0) {
                            e.id.clone()
                        } else {
                            String::new()
                        },
                    })
                    .collect::<Vec<MangaBasicInfo>>(),
            )
        }
    }
    #[allow(dead_code)]
    fn all_info(&self) -> Vec<MangaBasicInfo> {
        if self.all_basic_info.lock().unwrap().is_none() {
            self.collect_info();
        };
        self.all_basic_info.lock().unwrap().clone().unwrap()
    }
    pub fn all_json(&self) -> String {
        self.collect_info();
        serde_json::to_string(match &*self.all_basic_info.lock().unwrap() {
            Some(v) => v,
            None => panic!("??!"),
        })
        .unwrap()
    }

    pub fn get_list_mut(
        &self,
    ) -> impl std::ops::DerefMut<Target = HashMap<String, MangaInfo>> + '_ {
        self.list.lock().unwrap()
    }
}

lazy_static::lazy_static! {
    static ref MANGA_LIST :MangaList = single_mangalist();
}

#[test]
fn t_s() {
    #[derive(Debug, serde::Deserialize, serde::Serialize)]
    struct T {
        a: usize,
        b: String,
        c: Vec<usize>,
        d: Option<usize>,
        e: Option<usize>,
    }
    println!(
        "{}",
        toml::to_string(&T {
            a: 32,
            b: "ddd".to_string(),
            c: Vec::new(),
            d: None,
            e: Some(4),
        })
        .unwrap()
    );

    let s = "a = 32\n    b = \"ddd\" \n    c = [32]\n    e = 4";

    let un: T = toml::from_str(s).unwrap();
    dbg!(un);
}

fn single_mangalist() -> MangaList {
    match CONFIG.backend {
        Backend::DMZJ => dmzj::Dmzj::generate_manga_list(),
        Backend::CopyManga => copy_manga::CopyManga::generate_manga_list(),
        Backend::Eh => eh::Eh::generate_manga_list(),
    }
}

pub fn get_list_ref() -> &'static MangaList {
    &MANGA_LIST
}
lazy_static::lazy_static!(
   pub static ref CONFIG: Config = config();
);
#[derive(Debug, Clone, Copy, serde::Deserialize, serde::Serialize)]
pub enum Backend {
    DMZJ,
    CopyManga,
    Eh,
}
#[derive(Debug, Clone, Copy, serde::Deserialize)]
pub struct Config {
    pub port: u16,
    pub backend: Backend,
}
fn config() -> Config {
    #[derive(Debug, Clone, serde::Deserialize)]
    struct ConfigD {
        port: u16,
        backend: String,
    }
    let f = std::fs::read_to_string("config.toml").unwrap();
    let config: ConfigD = toml::from_str(&f).unwrap();
    let backend = match config.backend {
        v if v == "dmzj" => Backend::DMZJ,
        v if v == "copy_manga" => Backend::CopyManga,
        v if v == "eh" => Backend::Eh,
        _ => panic!(),
    };
    Config {
        port: config.port,
        backend,
    }
}
#[test]
fn t() {
    let p = r"H:\g\Books\manga\zips";
    let mut c = 0;
    let w = walkdir::WalkDir::new(p);
    let mut h: HashMap<u64, usize> = HashMap::new();
    for i in w {
        let i = match i {
            Ok(i) => i,
            Err(_) => continue,
        };
        if i.file_type().is_file() {
            let name = i.file_name().to_str().unwrap().to_owned();
            let id: u64 = name.split("_").next().unwrap().parse().unwrap();
            if h.contains_key(&id) {
                *h.get_mut(&id).unwrap() += 1;
            } else {
                h.insert(id, 1);
            }
            c += 1;
        };
    }

    for (k, v) in &h {
        println!("{:5} {:5}", k, v);
    }
    println!("{}", c);
    println!("{}", h.len());
}
