#![feature(iterator_try_collect)]
#![feature(never_type)]
#![feature(step_trait)]
#![feature(trait_alias)]

pub(crate) mod error;
pub mod hive;
pub mod searchargs;
pub mod strategy;
pub(crate) mod uhp;

#[allow(unused)]
pub mod prelude
{
    pub use std::str::FromStr;

    pub use log::{self};

    pub use crate::{
        error::{Error, Kind, Result},
        hive::*,
        searchargs::*,
        strategy::*,
        uhp::{Server, UhpOptions},
    };
}
