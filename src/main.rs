//! Substrate Node Template CLI library.

#![warn(missing_docs)]
#![warn(unused_extern_crates)]

mod chain_spec;
mod service;
mod cli;

pub use substrate_cli::{VersionInfo, IntoExit, error};

fn run() -> cli::error::Result<()> {
	let version = VersionInfo { name: "Substrate Node",
	                            commit: env!("VERGEN_SHA_SHORT"),
	                            version: env!("CARGO_PKG_VERSION"),
	                            executable_name: "akropolis-c2fc",
	                            author: "AkropolisRnDTeam",
	                            description: "Akropolis C2FC",
	                            support_url: "support.akropolis.io" };
	cli::run(::std::env::args(), cli::Exit, version)
}

error_chain::quick_main!(run);
