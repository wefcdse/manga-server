use std::{
    collections::HashMap,
    io::{BufRead, Write},
};

use async_zip::{tokio::read::ZipEntryReader, ZipFile, ZipFileBuilder};

pub async fn get_manga_pic() -> anyhow::Result<Vec<u8>> {
    todo!()
}

pub async fn get_pic_in_zip(
    bass_path: &str,
    manga_id: &str,
    hua: &str,
    pic_id: usize,
) -> anyhow::Result<Option<Vec<u8>>> {
    use async_zip::tokio::read::seek::ZipFileReader;
    use tokio::{fs::File, io::AsyncReadExt};
    let mut file = File::open(&format!(r"{}/{}_{}.zip", bass_path, manga_id, hua)).await?;
    let mut zip = ZipFileReader::with_tokio(&mut file).await?;
    let f = zip.file();

    let pic_file = format!("{}.jpg", pic_id);

    let e = f
        .entries()
        .iter()
        .filter(|e| e.entry().filename().as_str().unwrap() == pic_file)
        .collect::<Vec<_>>();

    let e = {
        let mut a = None;
        for (id, e) in f.entries().iter().enumerate() {
            if e.entry().filename().as_str()? == pic_file {
                a = Some(id);
                break;
            };
        }
        match a {
            Some(v) => v,
            None => return Ok(None),
        }
    };

    let mut reader = zip.reader_with_entry(e).await?;
    let mut out = Vec::new();
    let _ = reader.read_to_end_checked(&mut out).await?;
    Ok(Some(out))
}

pub async fn get_zip_length(bass_path: &str, manga_id: &str, hua: &str) -> anyhow::Result<usize> {
    use async_zip::tokio::read::seek::ZipFileReader;
    use tokio::fs::File;
    let mut file = File::open((&format!(r"{}/{}_{}.zip", bass_path, manga_id, hua))).await?;
    let zip = ZipFileReader::with_tokio(&mut file).await?;
    let f = zip.file();

    Ok(f.entries().len())
}

pub fn read_id_mapping(path: &str) -> HashMap<usize, String> {
    use std::fs;
    let mut f = fs::OpenOptions::new().read(true).open(path).unwrap();

    use std::io::BufReader;
    let mut f = BufReader::new(f);
    let mut id_mapping = HashMap::new();
    loop {
        let mut s = String::new();
        let line = f.read_line(&mut s).unwrap();
        if line == 0 {
            break;
        }
        let mut s = s.split("==>");
        let id = s.next().unwrap().parse::<usize>().unwrap();
        let name = s
            .next()
            .unwrap()
            .chars()
            .filter(|c| !c.is_whitespace())
            .collect::<String>()
            .to_owned();
        //println!("{}:{}", id, name);
        id_mapping.insert(id, name);
    }
    id_mapping
}

pub fn read_all_zips(path: &str) -> HashMap<usize, Vec<usize>> {
    let w = walkdir::WalkDir::new(path);
    let mut o: HashMap<usize, Vec<usize>> = HashMap::new();
    for e in w {
        let e = e.unwrap();
        if e.file_type().is_file() {
        } else {
            continue;
        }
        let mut s = e.file_name().to_str().unwrap().split('_');
        let id1 = s.next().unwrap().parse().unwrap();
        let id2 = s
            .next()
            .unwrap()
            .split('.')
            .next()
            .unwrap()
            .parse()
            .unwrap();
        if let Some(v) = o.get_mut(&id1) {
            v.push(id2);
        } else {
            o.insert(id1, vec![id2]);
        }
    }
    for (_k, v) in o.iter_mut() {
        v.sort();
    }
    o
}

#[test]
fn t_read_all_zips() {
    let p = r"H:\g\Books\manga\zips";
    dbg!(read_all_zips(p));
}

#[test]
fn generate_id_mapping() {
    let p = r"H:\g\Books\manga\mapping";
    let mut c = 0;
    let w = walkdir::WalkDir::new(p);
    let mut h: HashMap<u64, String> = HashMap::new();
    for i in w {
        let i = match i {
            Ok(i) => i,
            Err(_) => continue,
        };
        if i.file_type().is_file() {
            let name = i.file_name().to_str().unwrap().to_owned();
            let id: u64 = name.split(" ").next().unwrap().parse().unwrap();
            let name = {
                let mut n = name.split(" ");
                n.next();
                n.next()
            }
            .unwrap()
            .split(".")
            .next()
            .unwrap()
            .to_owned();
            if h.contains_key(&id) {
                panic!()
            } else {
                h.insert(id, name);
            }
            c += 1;
        };
    }
    use std::fs;
    let mut f = fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .create_new(true)
        .open("mapping.txt")
        .unwrap();
    let mut a: Vec<u64> = h.iter().map(|(k, _)| *k).collect();
    a.sort();
    let b = a
        .iter()
        .map(|id| (*id, h.get(id).unwrap().to_owned()))
        .collect::<Vec<_>>();

    for (k, v) in &b {
        write!(f, "{}==>{}\n", k, v).unwrap();
        println!("{:5} {:5}", k, v);
    }
    println!("{}", c);
    println!("{}", h.len());
    println!("{:?}", b);
}

#[test]
fn t() {
    dbg!(read_id_mapping("mapping.txt"));
}
#[test]
fn t_unzip() {
    let a = async {
        use async_zip::tokio::read::seek::ZipFileReader;
        use tokio::{fs::File, io::AsyncReadExt};
        let mut file = File::open("./test.zip").await.unwrap();

        let mut zip = ZipFileReader::with_tokio(&mut file).await.unwrap();

        let mut string = String::new();

        let f = zip.file();
        for e in f.entries() {
            if !e.entry().dir().unwrap() {
                continue;
            };
            println!("{}", e.entry().filename().as_str().unwrap());
        }

        let mut reader = zip.reader_with_entry(1).await.unwrap();
        let l = reader.read_to_string_checked(&mut string).await.unwrap();

        //println!("{}", string);
    };
    let rt = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    rt.block_on(a);
}
