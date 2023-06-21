use crate::manga_list::MangaList;

#[derive(Debug, Clone, serde::Serialize)]
pub struct ChapterInfo {
    pub length: usize,
    pub name: String,
}

#[async_trait::async_trait]
pub trait BackendTrait {
    fn generate_manga_list() -> MangaList;
    async fn get_pic_in_chapter(
        base_path: &str,
        manga_id: &str,
        chapter: &str,
        pic_id: usize,
    ) -> anyhow::Result<Option<Vec<u8>>>;
    async fn get_chapter_info(
        base_path: &str,
        manga_id: &str,
        chapter: &str,
    ) -> anyhow::Result<ChapterInfo>;
}

#[test]
fn t_walkdir() {
    let w = walkdir::WalkDir::new(r"D:\aaa");
    for e in w {
        let e = e.unwrap();
        // dbg!(e);
        //dbg!(e.depth());
        if e.depth() == 1 {
            dbg!(e);
        }
    }
}
