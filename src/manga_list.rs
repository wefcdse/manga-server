use std::{
    collections::HashMap,
    io::{BufRead, BufReader},
    sync::{Arc, Mutex},
};

use lazy_static::__Deref;
use serde::Serialize;

use crate::dmzj::{read_all_zips, read_id_mapping};

#[derive(Debug, Serialize, PartialEq, Clone, Eq)]
pub struct MangaInfo {
    pub name: String,
    pub pic: String,
    pub id: String,
    pub zips: Vec<usize>,
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
                        first: if let Some(e) = v.zips.get(0) {
                            e.to_string()
                        } else {
                            String::new()
                        },
                    })
                    .collect::<Vec<MangaBasicInfo>>(),
            )
        }
    }
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

    pub fn get_list_mut<'a>(
        &'a self,
    ) -> impl std::ops::DerefMut<Target = HashMap<String, MangaInfo>> + 'a {
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
    };
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
    let f = std::fs::OpenOptions::new()
        .read(true)
        .open("path.txt")
        .unwrap();
    let mut bf = BufReader::new(f);
    let mut p1 = String::new();
    let mut p2 = String::new();
    bf.read_line(&mut p1).unwrap();
    bf.read_line(&mut p2).unwrap();

    let p1 = p1
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect::<String>();
    let p2 = p2
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect::<String>();

    let l = MangaList::new(&p1);
    let mapping = read_id_mapping(&p2);
    let all = read_all_zips(&p1);

    for (k, v) in all {
        if let Some(name) = mapping.get(&k) {
            l.get_list_mut().insert(
                k.to_string(),
                MangaInfo {
                    name: name.to_owned(),
                    pic: format!("/manga/{}/{}/{}", k, v[0], 0),
                    id: k.to_string(),
                    zips: v,
                },
            );
        } else {
            l.get_list_mut().insert(
                k.to_string(),
                MangaInfo {
                    name: k.to_string(),
                    pic: format!("/manga/{}/{}/{}", k, v[0], 0),
                    id: k.to_string(),
                    zips: v,
                },
            );
        }
    }

    l
}

pub fn get_list_ref() -> &'static MangaList {
    &MANGA_LIST
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
