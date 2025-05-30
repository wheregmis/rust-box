#[macro_use]
extern crate serde;

pub mod transferpb {
    include!("transferpb.rs");
}

pub type Priority = u32;
pub(crate) type Id = u64;
pub mod client;
pub mod server;

pub use anyhow::{Error, Result};
