use std::{
	ffi::OsStr,
	fs::{self, OpenOptions},
	io::{self, Write},
	path::{Path, PathBuf},
};

use crate::{
	b2u32,
	cli::Cli,
	sprite::{Align, Sprite, TileMap},
};
use anyhow::{Result, bail, ensure};
use image::{
	ExtendedColorType, ImageEncoder, Rgba, RgbaImage,
	codecs::png::{CompressionType, FilterType, PngEncoder},
};
use tap::prelude::*;

pub fn dump_cg(args: Cli) -> Result<()> {
	let path = PathBuf::from(args.path);

	ensure!(
		path.extension().map(|e| e.eq("cg")).unwrap_or(false),
		"File to be dumped must have .cg extension"
	);

	let data = read_cg(&path)?;
	let out_path = output_folder(&path)?;
	let pal = dump_pal(&data, &out_path)?;
	dump_sprites(&data, pal, &out_path)?;

	Ok(())
}

fn dump_pal<'a>(data: &'a [u8], out_path: impl AsRef<Path>) -> Result<&'a [u8]> {
	let out_path = out_path.as_ref();
	let out_path = PathBuf::from(out_path)
		.tap_mut(|a| a.push(out_path.file_name().unwrap()))
		.tap_mut(|a| {
			a.set_extension("pal");
		});

	let out_pal = &data[20..(20 + 4 * 16 * 16)]; // WTF

	let mut file = OpenOptions::new()
		.create(true)
		.truncate(true)
		.write(true)
		.open(out_path)?;

	file.write_all(&0x40u32.to_le_bytes())?;
	file.write_all(&data[20..(0x2000 + 20)])?;

	Ok(out_pal)
}

fn dump_sprites(data: &[u8], pal: &[u8], out_path: &OsStr) -> Result<()> {
	let d = 16 + 1 * 4 + 0x800 * 4;
	let indices = d + 12 * 4;

	let num_sheets = b2u32(&data[d..d + 4]);
	ensure!(b2u32(&data[d + 4..d + 8]) == 0);
	let num_aligns = b2u32(&data[d + 8..d + 12]);

	let image_count = b2u32(&data[d + 12..d + 16]) as usize;

	let align_index = b2u32(&data[(indices + 3000 * 4)..(indices + 3001 * 4)]) as usize;
	let m_align = &data[align_index..];
	let m_indices = &data[indices..];
	let index = b2u32(&m_indices[..4]) as usize;

	let sprite = Sprite::new(&data[index..], index);
	let mut tm = TileMap::new(num_sheets, num_aligns, sprite.width);
	for n in 0..image_count {
		let index = b2u32(&m_indices[n * 4..(n + 1) * 4]) as usize;
		let sprite = Sprite::new(&data[index..], index);

		println!("sprite.filename: {}", &sprite.filename);
		println!("n: {}", n);
		draw_sprite(&sprite, m_align, pal, &mut tm, out_path)?;
	}

	Ok(())
}

fn read_cg(path: impl AsRef<Path>) -> Result<Vec<u8>> {
	Ok(fs::read(path)?)
}

fn output_folder(path: &impl AsRef<Path>) -> Result<&OsStr> {
	let output_path = path.as_ref().file_prefix().unwrap();

	match fs::create_dir(output_path) {
		Ok(_) => Ok(output_path),
		Err(e) => match e.kind() {
			io::ErrorKind::AlreadyExists => Ok(output_path),
			x => bail!("Error creating the output directory: {x}"),
		},
	}
}

fn draw_sprite(
	sprite: &Sprite,
	m_align: &[u8],
	pal: &[u8],
	tm: &mut TileMap,
	out_path: &OsStr,
) -> Result<()> {
	let mut image = RgbaImage::new(sprite.width, sprite.height);

	for p in image.pixels_mut() {
		if sprite.type_id == 0 {
			*p = Rgba([pal[0], pal[1], pal[2], 255]);
		} else {
			*p = Rgba([0, 0, 0, 0]);
		}
	}

	println!("sprite.value.len: {}", sprite.values.len());
	println!("sprite.type_id: {}", sprite.type_id);

	let mut j = 0;
	for i in 0..sprite.align_len {
		let i = i as usize;
		let align = Align::new(sprite, m_align, i);
		let nval = align.width * align.height;

		tm.reserve(nval as usize);
		for e in 0..nval {
			let source_xval = align.source_x + (e % align.width);
			let source_yval = align.source_y + e / align.width;

			let pix = if !align.backref {
				let color = sprite.next_color(pal, &mut j)?;
				tm.set(align.source_img, source_xval, source_yval, color);
				j += 1;

				color
			} else {
				tm.get(align.source_img, source_xval, source_yval)
			};

			let xval = align.x + e % align.width;
			let yval = align.y + e / align.width;
			image.put_pixel(xval, yval, pix);
		}

		if sprite.type_id == 4 && !align.backref {
			for e in 0..nval {
				let dval = sprite.values[j];

				let source_xval = align.source_x + e % align.width;
				let source_yval = align.source_y + e / align.width;
				tm.set_alpha(align.source_img, source_xval, source_yval, dval);

				let xval = align.x + e % align.width;
				let yval = align.y + e / align.width;
				image.get_pixel_mut(xval, yval).0[3] = dval;

				j += 1;
			}
		}
	}

	let mut path = PathBuf::from(out_path)
		.tap_mut(|p| p.push(&*sprite.filename))
		.tap_mut(|p| {
			p.set_extension("bmp.png");
		});

	let f = OpenOptions::new()
		.create(true)
		.truncate(true)
		.write(true)
		.open(&path)?;

	let e = PngEncoder::new_with_quality(f, CompressionType::Fast, FilterType::Adaptive);
	e.write_image(
		&image,
		image.width(),
		image.height(),
		ExtendedColorType::Rgba8,
	)?;

	if sprite.type_id == 2 || sprite.type_id == 4 {
		ensure!(sprite.cpal.len() == 1024);

		path.set_extension("cpal");

		let mut f = OpenOptions::new()
			.create(true)
			.truncate(true)
			.write(true)
			.open(&path)?;

		f.write_all(sprite.cpal)?;

		if sprite.type_id == 4 {
			path.set_extension("t4");

			let mut f = OpenOptions::new()
				.create(true)
				.truncate(true)
				.write(true)
				.open(&path)?;

			f.write_all(b"4")?;
		}
	} else if sprite.type_id == 3 {
		path.set_extension("t4");

		let mut f = OpenOptions::new()
			.create(true)
			.truncate(true)
			.write(true)
			.open(&path)?;

		f.write_all(format!("{} {} {}", sprite.r, sprite.g, sprite.b).as_bytes())?;
	}

	Ok(())
}
