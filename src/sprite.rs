use std::collections::HashMap;

use bytes::Buf;
use encoding_rs::SHIFT_JIS;
use image::Rgba;
use tap::prelude::*;

#[derive(Default)]
pub struct Sprite {
	pub index: u32,
	pub filename: String,
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
	pub values: Vec<u8>,
	pub cpal: Vec<u8>,
}

impl Sprite {
	pub fn new(mut data: &[u8], index: usize) -> Self {
		let mut s = Sprite::default();
		s.index = index as u32;

		s.filename = SHIFT_JIS
			.decode(&data[..32])
			.0
			.pipe(|x| x.trim_matches('\0').to_string());

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
				s.values = data[..((s.width * s.height * 4) as usize)].to_vec();
			}
			2 => {
				s.cpal = data[..1024].to_vec();
				data = &data[1024..];

				s.values = data[..((s.width * s.height) as usize)].to_vec();
			}
			3 => {
				s.r = data.get_u8();
				s.g = data.get_u8();
				s.b = data.get_u8();
				s.a = data.get_u8();

				s.values = data[..((s.width * s.height) as usize)].to_vec();
			}
			4 => {
				s.cpal = data[..1024].to_vec();
				data = &data[1024..];

				s.values = data[..((s.width * s.height * 2) as usize)].to_vec();
			}
			_ => {
				s.values = data[..((s.width * s.height - 72) as usize)].to_vec();
			}
		}

		s
	}
}

pub struct TileMap {
	pub tiles: HashMap<u32, Vec<HashMap<u32, u8>>>,
	pub alphatiles: HashMap<u32, Vec<HashMap<u32, u8>>>,
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

	pub fn set(&mut self, sheet: u32, x: u32, y: u32, r: u8, g: u8, b: u8, a: u8) {
		if self.mode == 0 {
			match self.tiles.entry(sheet) {
				std::collections::hash_map::Entry::Occupied(mut view) => {
					view.get_mut()[0].insert(x + y * 256, r);
					view.get_mut()[1].insert(x + y * 256, g);
					view.get_mut()[2].insert(x + y * 256, b);
					view.get_mut()[3].insert(x + y * 256, a);
				}
				std::collections::hash_map::Entry::Vacant(view) => {
					let mut new = vec![HashMap::new(); 4];

					new[0].insert(x + y * 256, r);
					new[1].insert(x + y * 256, g);
					new[2].insert(x + y * 256, b);
					new[3].insert(x + y * 256, a);

					view.insert(new);
				}
			}
		} else {
			match self.alphatiles.entry(sheet) {
				std::collections::hash_map::Entry::Occupied(mut view) => {
					view.get_mut()[0].insert(x + y * 256, r);
					view.get_mut()[1].insert(x + y * 256, g);
					view.get_mut()[2].insert(x + y * 256, b);
					view.get_mut()[3].insert(x + y * 256, a);
				}
				std::collections::hash_map::Entry::Vacant(view) => {
					let mut new = vec![HashMap::new(); 4];

					new[0].insert(x + y * 256, r);
					new[1].insert(x + y * 256, g);
					new[2].insert(x + y * 256, b);
					new[3].insert(x + y * 256, a);

					view.insert(new);
				}
			}
		}
	}

	pub fn set_alpha(&mut self, sheet: u32, x: u32, y: u32, a: u8) {
		self.alphatiles.entry(sheet).and_modify(|v| {
			v[3].insert(x + y * 256, a);
		});
	}

	pub fn get(&self, sheet: u32, x: u32, y: u32) -> Rgba<u8> {
		let r = self.tiles[&sheet][0][&(x + y * 256)];
		let g = self.tiles[&sheet][1][&(x + y * 256)];
		let b = self.tiles[&sheet][2][&(x + y * 256)];
		let a = self.tiles[&sheet][3][&(x + y * 256)];

		Rgba([r, g, b, a])
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
	pub backref: u32,
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
			backref: data.get_u16_le() as u32,
		}
	}
}
