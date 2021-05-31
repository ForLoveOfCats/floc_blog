use std::fs::canonicalize;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;

mod arguments;

#[derive(Debug)]
struct Fragments {
	css: String,
	header: String,
	footer: String,
}

impl Fragments {
	fn retrive_or_shim(dir: Option<PathBuf>) -> Fragments {
		let mut dir = match dir {
			Some(dir) => dir,

			None => {
				return Fragments {
					css: String::new(),
					header: String::new(),
					footer: String::new(),
				};
			}
		};

		fn get_fragment(dir: &mut PathBuf, name: &str) -> String {
			dir.push(name);

			let fragment = match std::fs::read_to_string(&dir) {
				Ok(fragment) => fragment,

				Err(err) => {
					eprintln!("Error loading fragment '{}': {}", name, err);
					std::process::exit(-1);
				}
			};

			dir.pop();
			fragment
		}

		let css = get_fragment(&mut dir, "style.css");
		let header = get_fragment(&mut dir, "header.html");
		let footer = get_fragment(&mut dir, "footer.html");

		Fragments {
			css,
			header,
			footer,
		}
	}
}

fn process_file(
	input_buffer: &mut String,
	path: PathBuf,
	output_path: PathBuf,
	_fragments: &Fragments,
) {
	let is_markdown = path.extension().map(|p| p.to_str()) == Some(Some("md"));

	if !is_markdown {
		if let Some(dir_path) = output_path.parent() {
			/*
			 * NOTE: Silently swallow failure to create output path.
			 * If the path does not exist the copy will still catch
			 * the error. Otherwise if this failed for some other
			 * reason but the copy can still succeed then we do not
			 * care that this failed.
			 */
			let _ = std::fs::create_dir_all(dir_path);
		}

		if let Err(err) = std::fs::copy(&path, &output_path) {
			eprintln!(
				"Error copying input file '{}' to '{}': {}",
				path.to_string_lossy(),
				output_path.to_string_lossy(),
				err
			);
			std::process::exit(-1);
		}
	} else {
		let mut file = match File::open(&path) {
			Ok(file) => file,

			Err(err) => {
				eprintln!(
					"Error reading input file '{}': {}",
					path.to_string_lossy(),
					err
				);
				std::process::exit(-1);
			}
		};

		input_buffer.clear();
		if let Err(err) = file.read_to_string(input_buffer) {
			eprintln!(
				"Error reading input markdown file '{}': {}",
				path.to_string_lossy(),
				err
			);
			std::process::exit(-1);
		}

		println!("read markdown source");
	}
}

fn main() {
	let args = arguments::parse();

	let fragments = Fragments::retrive_or_shim(args.fragments_dir);

	let canonical_input_dir = match canonicalize(&args.input_dir) {
		Ok(canonical_input_dir) => canonical_input_dir,

		Err(err) => {
			eprintln!(
				"Error canoncializing input dir path '{}': {}",
				args.input_dir.to_string_lossy(),
				err
			);
			std::process::exit(-1);
		}
	};
	let canonical_input_dir_component_count = canonical_input_dir.components().count();

	let input_dir = match std::fs::read_dir(&args.input_dir) {
		Ok(input_dir) => input_dir,

		Err(err) => {
			eprintln!(
				"Error opening input dir '{}': {}",
				args.input_dir.to_string_lossy(),
				err
			);
			std::process::exit(-1);
		}
	};

	let mut stack = vec![input_dir];
	let mut input_buffer = String::new();
	while let Some(top) = stack.last_mut() {
		match top.next() {
			Some(Ok(entry)) => {
				let path = entry.path();

				let is_file = entry.file_type().map(|e| e.is_file()).unwrap_or(false);
				let is_dir = entry.file_type().map(|e| e.is_dir()).unwrap_or(false);

				if is_file {
					let canonical_file_path = match canonicalize(&path) {
						Ok(canonical_file_path) => canonical_file_path,

						Err(err) => {
							eprintln!(
								"Error canoncializing file path '{}': {}",
								path.to_string_lossy(),
								err
							);
							std::process::exit(-1);
						}
					};

					let trailing_path = canonical_file_path
						.components()
						.skip(canonical_input_dir_component_count)
						.collect::<PathBuf>();

					let output_path = {
						let mut output_path = args.output_dir.clone();
						output_path.push(trailing_path);
						output_path
					};

					process_file(&mut input_buffer, path, output_path, &fragments);
				} else if is_dir {
					if let Ok(sub) = std::fs::read_dir(path) {
						stack.push(sub);
						continue;
					}
				}
			}

			Some(Err(err)) => {
				eprintln!("Error walking input dir: {}", err);
				std::process::exit(-1);
			}

			None => {
				stack.pop();
			}
		}
	}
}
