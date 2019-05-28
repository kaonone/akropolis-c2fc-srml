//! Akropolis Substrate Node.

#![warn(missing_docs)]
#![warn(unused_extern_crates)]

#[macro_use]
extern crate hex_literal;

mod consts;
// mod error;
mod chain_spec;
mod service;
mod cli;

pub use substrate_cli::{VersionInfo, IntoExit, error};

fn main() {
	use consts::*;

	let version = VersionInfo {
		name: NODE_NAME,
		commit: env!("VERGEN_SHA_SHORT"),
		version: env!("CARGO_PKG_VERSION"),
		executable_name: "akropolis",
		author: AUTHOR_NAME,
		description: DESCRIPTION,
		support_url: SUPPORT_URL,
	};

	if let Err(e) = cli::run(::std::env::args(), cli::Exit, version) {
		eprintln!("Error starting the node: {}\n\n{:?}", e, e);
		std::process::exit(1)
	}
}
