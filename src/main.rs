use std::fs::File;
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

fn process_file(input_buffer: &mut String, path: PathBuf, _fragments: &Fragments) {
	input_buffer.clear();

	let _file = match File::open(&path) {
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

	println!("opened file at path '{}'", path.to_string_lossy());
}

fn main() {
	let args = arguments::parse();

	let fragments = Fragments::retrive_or_shim(args.fragments_dir);

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
				let is_file = entry.file_type().map(|e| e.is_file()).unwrap_or(false);
				let is_dir = entry.file_type().map(|e| e.is_dir()).unwrap_or(false);

				if is_file {
					process_file(&mut input_buffer, entry.path(), &fragments);
				} else if is_dir {
					if let Ok(sub) = std::fs::read_dir(entry.path()) {
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
