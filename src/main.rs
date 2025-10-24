mod cli;
mod dump;
mod sprite;
mod warning;

use std::process::ExitCode;

use crate::{cli::Cli, dump::dump_cg, warning::warning_printed};
use anyhow::Result;
use clap::Parser;

enum Return {
	Ok = 0,
	Error = 1,
	Warning = 2,
}

fn main() -> ExitCode {
	match run() {
		Ok(_) => match warning_printed() {
			true => ExitCode::from(Return::Warning as u8),
			false => ExitCode::from(Return::Ok as u8),
		},
		Err(e) => {
			eprintln!("{:?}", e);
			ExitCode::from(Return::Error as u8)
		}
	}
}

fn run() -> Result<()> {
	let args = cli::Cli::parse(); // CLI arguments

	match args.operation {
		cli::Operation::Dump => dump_cg(args),
		cli::Operation::Pack => pack_cg(args),
	}
}

fn pack_cg(args: Cli) -> Result<()> {
	todo!()
}

pub fn b2u32(slice: &[u8]) -> u32 {
	u32::from_le_bytes(slice.try_into().unwrap())
}
