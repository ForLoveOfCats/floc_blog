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

fn main() {
	let args = arguments::parse();

	let fragments = Fragments::retrive_or_shim(args.fragments_dir);
	println!("{:#?}", fragments);
}
