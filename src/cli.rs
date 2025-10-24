use std::ffi::OsString;

use clap::{Parser, ValueEnum};

#[derive(Debug, Parser)]
#[clap(version)]
pub struct Cli {
	/// File or folder to process
	pub path: OsString,

	/// Operation to do on the path
	#[arg(short, long, value_enum, default_value_t = {Operation::Dump})]
	pub operation: Operation,

	/// Useless [WIP]
	#[arg(short, long, value_enum, default_value_t = {TileWidth::P16})]
	pub tile_width: TileWidth,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum Operation {
	Dump,
	Pack,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum TileWidth {
	P16,
	P32,
}
