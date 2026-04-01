use anyhow::{Context, Result, bail};
use image::GenericImageView;
use std::fs;

pub fn load_cubemap(path: &str) -> Result<()> {
	let input_path = std::path::Path::new(path);
	if !input_path.exists() {
		bail!("Cubemap source file does not exist: {}", input_path.display());
	}

	let stem = input_path
		.file_stem()
		.and_then(|s| s.to_str())
		.context("Cubemap source path must have a valid UTF-8 file stem")?;

	let output_dir = input_path
		.parent()
		.unwrap_or_else(|| std::path::Path::new("."))
		.join(format!("{stem}_cubemap_faces"));

	fs::create_dir_all(&output_dir)
		.with_context(|| format!("Failed to create output directory `{}`", output_dir.display()))?;

	let image = image::open(path)
		.with_context(|| format!("Failed to open cubemap cross image at `{path}`"))?
		.to_rgba8();

	let width = image.width();
	let height = image.height();

	if width % 4 != 0 || height % 3 != 0 {
		bail!(
			"Cubemap cross image must have a 4:3 grid layout, got {}x{}",
			width,
			height
		);
	}

	let face_size = width / 4;
	if height / 3 != face_size {
		bail!(
			"Cubemap cross image must use square faces, got {}x{} with face size {}x{}",
			width,
			height,
			face_size,
			height / 3
		);
	}

	// Cross layout:
	//       +Y
	// -X +Z +X -Z
	//       -Y
	// Saved names:
	// right(+X), left(-X), top(+Y), bottom(-Y), front(+Z), back(-Z)
	let faces = [
		("right", 2, 1),
		("left", 0, 1),
		("top", 1, 0),
		("bottom", 1, 2),
		("front", 1, 1),
		("back", 3, 1),
	];

	for (name, grid_x, grid_y) in faces {
		let face = image
			.view(grid_x * face_size, grid_y * face_size, face_size, face_size)
			.to_image();

		let out_path = output_dir.join(format!("{name}_{stem}.png"));
		if out_path.exists() {
			bail!(
				"Refusing to overwrite existing cubemap face `{}`",
				out_path.display()
			);
		}

		let tmp_path = output_dir.join(format!("{name}_{stem}.tmp.png"));
		face
			.save(&tmp_path)
			.with_context(|| format!("Failed to save cubemap face temp file `{}`", tmp_path.display()))?;

		fs::rename(&tmp_path, &out_path).with_context(|| {
			format!(
				"Failed to move cubemap face temp file from `{}` to `{}`",
				tmp_path.display(),
				out_path.display()
			)
		})?;
	}

	Ok(())
}