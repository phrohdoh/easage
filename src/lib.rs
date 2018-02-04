//! `easage` provides programmatic manipulation of BIG archives.
//!
//! BIG files are an archive format used in many games published by Electronic Arts.
//!
//! The BIG format is conceptually similar to TAR. It has magic, a header, and data.
//!
//! Neither compressed nor encrypted BIG formats are supported by easage at this time.
//!
//! # Getting started
//!
//! Read the below examples then check out the `Archive` struct.
//!
//! # Error handling
//!
//! Errors are bubbled-up via the [`failure`](https://crates.io/crates/failure) crate into
//! the all-encompassing `Error` enum.
//!
//! All errors implement the `Display` trait so feel free to print away!
//!
//! <small>
//! Side note: I don't think I am using all the goodies `failure` has to offer.
//! Please let me know if you see something that can be improved.
//! </small>
//!
//! # Examples
//!
//! Read an archive from a file:
//!
//! ```rust,no_run
//! use easage::Archive;
//!
//! // This must be a type that implements `AsRef<Path>`.
//! let path = "path/to/your.big";
//!
//! let archive = match Archive::from_path(path) {
//!     Ok(archive) => archive,
//!     Err(e) => {
//!         eprintln!("{}", e);
//!         return;
//!     },
//! };
//! ```
//!
//! Getting data of a file that is inside of an archive:
//!
//! ```rust,no_run
//! use easage::Archive;
//!
//! let mut archive = Archive::from_path("path/to/your.big").unwrap();
//!
//! // This provides us with a lookup table so we don't
//! // have to read the header repeatedly.
//! let table = match archive.read_entry_metadata_table() {
//!     Ok(table) => table,
//!     Err(e) => {
//!         eprintln!("{}", e);
//!         return;
//!     },
//! };
//!
//! // NOTE: `table` is an easage::EntryInfoTable which
//! // you can `.iter()` over to inspect all entries.
//!
//! if let Some(data) = archive.get_bytes_via_table(&table, "your/entry/name.txt") {
//!     // data: &[u8]
//! }
//! ```
//!
//! Package a directory (recursively) into an archive:
//!
//! ```rust,no_run
//! use std::io::Write;
//! use easage::Kind;
//! use easage::packer::{self, Settings, EntryOrderCriteria};
//!
//! // This must be a type that implements `AsRef<Path>`.
//! let directory_to_pack = "path/to/a/directory";
//!
//! let settings = Settings {
//!     // Order the archive entries alphanumeric by filepath.
//!     entry_order_criteria: EntryOrderCriteria::Path,
//!
//!     // We do not want to strip any prefix in this example.
//!     strip_prefix: None,
//!
//!     // The "magic" identifier (this isn't important yet)
//!     kind: Kind::BigF,
//! };
//!
//! // Finally we can create our archive!
//! let archive = match packer::pack_directory(directory_to_pack, settings) {
//!     Ok(archive) => archive,
//!     Err(e) => {
//!         eprintln!("{}", e);
//!         std::process::exit(1);
//!     },
//! };
//!
//! // At this point you probably want to write `archive` to a file.
//! let data = archive.as_slice();
//!
//! use std::fs::OpenOptions;
//!
//! let mut file = OpenOptions::new()
//!     .create(true)
//!     .read(true)
//!     .write(true)
//!     .truncate(true)
//!     .open("my_archive.big")
//!     .expect("Failed to open file for writing.");
//!
//! file.write_all(data).expect("Failed to write data to the new file.");
//! ```

extern crate byteorder;
extern crate memmap;
extern crate owning_ref;
extern crate walkdir;

#[macro_use(Fail)]
extern crate failure;

mod archive;
pub use archive::{Kind, EntryInfoTable, EntryInfo, Archive};

pub mod packer;

mod error;
pub use error::{Result, Error};

#[cfg(test)]
#[macro_use]
extern crate assert_matches;