use std::borrow::Cow;

use anyhow::{Result, bail};
use bytes::Buf;
use encoding_rs::SHIFT_JIS;
use hashbrown::{HashMap, hash_map};
use image::Rgba;
use tap::prelude::*;

#[derive(Default)]
pub struct Sprite<'a> {
	pub index: u32,
	pub filename: Cow<'a, str>,
	pub type_id: i32,
	pub bounds_x1: u32,
	pub bounds_x2: u32,
	pub bounds_y1: u32,
	pub bounds_y2: u32,
	pub width: u32,
	pub height: u32,
	pub bpp: u32,
	pub align_start: u32,
	pub align_len: u32,
	pub r: u8,
	pub g: u8,
	pub b: u8,
	pub a: u8,
	pub values: &'a [u8],
	pub cpal: &'a [u8],
}

impl<'a> Sprite<'a> {
	pub fn new(mut data: &'a [u8], index: usize) -> Self {
		let mut s = Sprite::default();
		s.index = index as u32;

		s.filename = SHIFT_JIS.decode(&data[..32]).0;

		data = &data[32..];
		s.type_id = data.get_i32_le();

		s.width = data.get_u32_le();
		s.height = data.get_u32_le();
		s.bpp = data.get_u32_le();
		s.bounds_x1 = data.get_u32_le();
		s.bounds_y1 = data.get_u32_le();
		s.bounds_x2 = data.get_u32_le();
		s.bounds_y2 = data.get_u32_le();
		s.align_start = data.get_u32_le();
		s.align_len = data.get_u32_le();

		match s.type_id {
			1 => {
				s.values = &data[..((s.width * s.height * 4) as usize)];
			}
			2 => {
				s.cpal = data[..1024].into();
				data = &data[1024..];

				s.values = &data[..((s.width * s.height) as usize)];
			}
			3 => {
				s.r = data.get_u8();
				s.g = data.get_u8();
				s.b = data.get_u8();
				s.a = data.get_u8();

				s.values = &data[..((s.width * s.height) as usize)];
			}
			4 => {
				s.cpal = &data[..1024];
				data = &data[1024..];

				s.values = &data[..((s.width * s.height * 2) as usize)];
			}
			_ => {
				s.values = &data[..((s.width * s.height - 72) as usize)];
			}
		}

		s
	}

	pub fn next_color(&self, pal: &[u8], dval_index: &mut usize) -> Result<Rgba<u8>> {
		let dval = self.values[*dval_index] as usize;

		let res = if self.bpp == 8 {
			Rgba([pal[dval * 4], pal[dval * 4 + 1], pal[dval * 4 + 2], 255])
		} else if self.type_id == 1 {
			// 32bpp bgra
			let b = self.values[*dval_index];
			let g = self.values[*dval_index + 1];
			let r = self.values[*dval_index + 2];
			let a = self.values[*dval_index + 3];
			*dval_index += 3;
			Rgba([r, g, b, a])
		} else if self.type_id == 2 {
			// custom pallete no alpha
			Rgba([
				self.cpal[dval * 4],
				self.cpal[dval * 4 + 1],
				self.cpal[dval * 4 + 2],
				255,
			])
		} else if self.type_id == 3 {
			// eff based color
			Rgba([self.r, self.g, self.b, dval as u8])
		} else if self.type_id == 4 {
			// custom pallete alpha
			Rgba([
				self.cpal[dval * 4],
				self.cpal[dval * 4 + 1],
				self.cpal[dval * 4 + 2],
				255,
			])
		} else {
			bail!("unhandled type");
		};

		Ok(res)
	}
}

#[allow(unused)]
pub struct TileMap {
	pub tiles: HashMap<u32, HashMap<u32, Rgba<u8>>>,
	pub alphatiles: HashMap<u32, HashMap<u32, Rgba<u8>>>,
	pub num_sheets: u32,
	pub num_aligns: u32,
	pub tile_width: u32,
	pub current_sheet: u32,
	pub current_alpha_sheet: u32,
	pub current_x: u32,
	pub current_y: u32,
	pub current_xa: u32,
	pub current_ya: u32,
	pub tmap: Vec<u8>,
	pub zero_tiles: Vec<u8>,
	pub pal: Vec<u8>,
	pub mode: u32,
}

impl TileMap {
	pub fn new(num_sheets: u32, num_aligns: u32, tile_width: u32) -> Self {
		Self {
			tiles: HashMap::new(),
			alphatiles: HashMap::new(),
			num_sheets,
			num_aligns,
			tile_width,
			current_sheet: 0,
			current_alpha_sheet: 0,
			current_x: 0,
			current_y: 0,
			current_xa: 0,
			current_ya: 0,
			tmap: Vec::new(),
			zero_tiles: Vec::new(),
			pal: Vec::new(),
			mode: 0,
		}
	}

	pub fn set(&mut self, sheet: u32, x: u32, y: u32, color: Rgba<u8>) {
		let tmp = Self::coord_2_linear(x, y);

		match self.mode == 0 {
			true => match self.tiles.entry(sheet) {
				hash_map::Entry::Occupied(mut view) => {
					view.get_mut().insert(tmp, color);
				}
				hash_map::Entry::Vacant(view) => {
					let new = HashMap::new().tap_mut(|h| {
						h.insert(tmp, color);
					});

					view.insert(new);
				}
			},
			false => match self.alphatiles.entry(sheet) {
				hash_map::Entry::Occupied(mut view) => {
					view.get_mut().insert(tmp, color);
				}
				hash_map::Entry::Vacant(view) => {
					let new = HashMap::new().tap_mut(|h| {
						h.insert(tmp, color);
					});

					view.insert(new);
				}
			},
		}
	}

	pub fn set_alpha(&mut self, sheet: u32, x: u32, y: u32, a: u8) {
		let tmp = Self::coord_2_linear(x, y);

		// ??? Is defaulting to Rgba([0, 0, 0, a]) right?
		match self.alphatiles.entry(sheet) {
			hash_map::Entry::Occupied(mut view) => match view.get_mut().entry(tmp) {
				hash_map::Entry::Occupied(mut view) => {
					view.get_mut().0[3] = a;
				}
				hash_map::Entry::Vacant(view) => {
					view.insert(Rgba([0, 0, 0, a]));
				}
			},
			hash_map::Entry::Vacant(view) => {
				let new = HashMap::new().tap_mut(|h| {
					h.insert(tmp, Rgba([0, 0, 0, a]));
				});

				view.insert(new);
			}
		}
	}

	pub fn get(&self, sheet: u32, x: u32, y: u32) -> Rgba<u8> {
		let tmp = Self::coord_2_linear(x, y);

		self.tiles[&sheet][&tmp]
	}

	pub fn reserve(&mut self, amount: usize) {
		self.tiles.reserve(amount);
		self.alphatiles.reserve(amount);
	}

	#[inline(always)]
	fn coord_2_linear(x: u32, y: u32) -> u32 {
		x + y * 256
	}
}

pub struct Align {
	pub x: u32,
	pub y: u32,
	pub width: u32,
	pub height: u32,
	pub source_x: u32,
	pub source_y: u32,
	pub source_img: u32,
	pub backref: bool,
}

impl Align {
	const SIZE: usize = 24;

	pub fn new(sprite: &Sprite, m_align: &[u8], seq: usize) -> Self {
		let data_start = ((sprite.align_start as usize) + seq) * Self::SIZE;
		let mut data = &m_align[data_start..(data_start + Self::SIZE)];

		Self {
			x: data.get_u32_le(),
			y: data.get_u32_le(),
			width: data.get_u32_le(),
			height: data.get_u32_le(),
			source_x: data.get_u16_le() as u32,
			source_y: data.get_u16_le() as u32,
			source_img: data.get_u16_le() as u32,
			backref: data.get_u16_le() != 0,
		}
	}
}
