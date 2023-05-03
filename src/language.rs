use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct EggIdWrapper {
    pub id: egg::Id,
}

#[derive(Debug, PartialEq, Eq)]
pub struct EggIdParseErr;

impl std::str::FromStr for EggIdWrapper {
    type Err = EggIdParseErr;

    fn from_str(s: &str) -> Result<EggIdWrapper, Self::Err> {
        let mut hasher = DefaultHasher::new();
        s.hash(&mut hasher);
        let res = hasher.finish() as usize;
        let egg_id: egg::Id = res.into();
        Ok(EggIdWrapper { id: egg_id })
    }
}

pub fn egg_id_from_str(s: &str) -> Result<egg::Id, String> {
    let mut hasher = DefaultHasher::new();
    s.hash(&mut hasher);
    let res = hasher.finish() as usize;
    let egg_id: egg::Id = res.into();
    Ok(egg_id)
}
