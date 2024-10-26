use anyhow::*;
use fs_extra::copy_items;
use fs_extra::dir::CopyOptions;
use std::env;

fn main() -> Result<()> {
	// This tells cargo to rerun this script if something in /resources/ changes.
	println!("cargo:rerun-if-changed=resources/materials/*");
	println!("cargo:rerun-if-changed=resources/objects/*");
	println!("cargo:rerun-if-changed=resources/textures/*");
	println!("cargo:rerun-if-changed=resources/shaders/*");
	println!("cargo:rerun-if-changed=resources/data/*");
	println!("cargo:rerun-if-changed=resources/sounds/*");

	let out_dir = env::var("OUT_DIR")?;
	let mut copy_options = CopyOptions::new();
	copy_options.overwrite = true;
	let mut paths_to_copy = Vec::new();
	paths_to_copy.push("resources/materials/");
	paths_to_copy.push("resources/objects/");
	paths_to_copy.push("resources/textures/");
	paths_to_copy.push("resources/shaders/");
	paths_to_copy.push("resources/data/");
	paths_to_copy.push("resources/sounds/");

	copy_items(&paths_to_copy, out_dir, &copy_options)?;

	Ok(())
}