use std::{error::Error, fmt::Display};

pub trait ToResult<V, E> {
    fn to_result(self) -> Result<V, E>;
}
#[derive(Debug, Clone)]
pub struct ToResultErr {
    inner: String,
}
impl Error for ToResultErr {}
impl Display for ToResultErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}
impl<T> ToResult<T, ToResultErr> for Option<T> {
    fn to_result(self) -> Result<T, ToResultErr> {
        match self {
            Some(v) => Ok(v),
            None => Err(ToResultErr {
                inner: "unwrapping None value".to_string(),
            }),
        }
    }
}

#[test]
fn t() {
    use std::collections::hash_map::HashMap;
    #[derive(Debug)]
    struct S1 {
        i: u64,
    }
    let mut h = HashMap::<String, u8>::new();
    h.insert("a".to_string(), 4);
    h.insert("wa".to_string(), 4);
    h.insert("2".to_string(), 4);
    h.insert("32".to_string(), 4);
    println!("{}", serde_json::to_string(&h).unwrap());
    dbg!((0..100)
        .into_iter()
        .map(|i| (1, i * 10))
        .collect::<HashMap<_, _>>());
    impl<A> FromIterator<A> for S1 {
        fn from_iter<T: IntoIterator<Item = A>>(iter: T) -> Self {
            let mut s = S1 { i: 0 };
            for (i, _i) in iter.into_iter().enumerate() {
                s.i += i as u64;
            }
            s
        }
    }
    let _a = (0..1).into_iter().next();
    let _a = 0..=1;
    dbg!((10..=110).into_iter().collect::<S1>());
}

pub trait ShortUnwrap<T> {
    fn u(self) -> T;
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub enum SingleOrMany {
    Single,
    Many(usize),
}
impl SingleOrMany {
    pub fn is_single(&self) -> bool {
        match self {
            Self::Single => true,
            Self::Many(_) => false,
        }
    }
    pub fn is_many(&self) -> bool {
        !self.is_single()
    }
    pub fn add(&mut self, v: usize) {
        if let Self::Many(i) = self {
            *i += v;
        } else {
            *self = Self::Many(1 + v);
        }
    }
}

impl<T, E: Error> ShortUnwrap<T> for Result<T, E> {
    fn u(self) -> T {
        self.unwrap()
    }
}

impl<T> ShortUnwrap<T> for Option<T> {
    fn u(self) -> T {
        self.unwrap()
    }
}

#[test]
fn t1() {
    let mut n = -1;
    let mut sum = 0;
    for i in 1..=100 {
        sum += i * n;
        n *= -1;
    }
    dbg!(sum);
    let s = r"1827529-[Yuribatake Bokujou (Kon)] Otome Game no Heroine wo 3kai Ikasenaito Hametsu suru Heya ni Haitte Shimatta... Maria Uke Tsuika Patch (Otome Game no Hametsu Flag shika Nai Akuyaku Reijou ni Tensei shiteshimatta...) [Chinese] [Digital]".to_string();
    dbg!(s.len());
}
