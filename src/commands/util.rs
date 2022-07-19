use std::{error::Error, fmt::Display};

use serenity::model::id::UserId;

#[derive(Debug, Clone, Copy)]
pub enum Target {
    Uid(UserId),
    MaybeId(u64),
}

#[derive(Debug)]
pub struct NoId;
impl Display for NoId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "No Id")
    }
}
impl Error for NoId {}

#[derive(Debug)]
pub struct HahaNo;
impl Display for HahaNo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "haha no")
    }
}
impl Error for HahaNo {}
