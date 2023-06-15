#![feature(iter_order_by)]

use icu_collator::Numeric::On;
use icu_collator::{Collator, CollatorOptions};
use log::{debug, info};
use std::fmt::{Debug, Display, Formatter};
use std::ops::Deref;
use std::sync::Arc;

pub mod api;
mod filerow;
#[cfg(feature = "leptos")]
mod leptos;
#[cfg(feature = "yew")]
mod yew;

pub use self::leptos::*;
pub use crate::leptos::mount_to_body;

fn make_magnet_link(info_hash: &str) -> String {
    "magnet:?xt=urn:btih:".to_owned() + info_hash
}

pub type Result<T> = std::result::Result<T, CloneableError>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    GlooNet(#[from] gloo_net::Error),
    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),
}

#[derive(Clone, Debug)]
pub struct CloneableError(Arc<Error>);

impl<I> From<I> for CloneableError
where
    I: Into<Error>,
{
    fn from(value: I) -> Self {
        Self(Arc::new(value.into()))
    }
}

impl From<Arc<Error>> for CloneableError {
    fn from(value: Arc<Error>) -> Self {
        Self(value)
    }
}

impl Deref for CloneableError {
    type Target = Arc<Error>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for CloneableError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0, f)
    }
}

impl std::error::Error for CloneableError {}

pub(crate) fn new_collator() -> Collator {
    let mut options = CollatorOptions::new();
    options.numeric = Some(On);
    icu_collator::Collator::try_new_unstable(
        &icu_testdata::unstable(),
        &Default::default(),
        options,
    )
    .unwrap()
}
