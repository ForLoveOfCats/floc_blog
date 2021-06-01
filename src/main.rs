use std::fmt::Write;
use std::fs::canonicalize;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;

use pulldown_cmark::{html, Event, Options, Parser};

mod arguments;

use arguments::Arguments;

macro_rules! multiline {
	( $($line:expr)* ) => {
		concat!( $($line, "\n"),* )
	}
}

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

struct Buffers {
	input: String,
	html: String,
	output: String,
	title: String,
	description: String,
	author: String,
}

fn process_markdown(_fragments: &Fragments, args: &Arguments, buffers: &mut Buffers) {
	let mut options = Options::empty();
	options.insert(Options::ENABLE_TABLES);
	let parser = Parser::new_ext(&buffers.input, options);

	/*
	 * NOTE: Borrowing these here borrows just the field instead of the entire
	 * struct which allows the closure to have mutable access to these two fields
	 * while `html::push_html` writes to another field.
	 */
	let title_buffer = &mut buffers.title;
	title_buffer.clear();
	let description_buffer = &mut buffers.description;
	description_buffer.clear();
	let author_buffer = &mut buffers.author;
	author_buffer.clear();

	let parser = parser.map(|event| {
		if let Event::Html(html) = &event {
			let html = html.trim();
			if html.starts_with("<!--") && html.ends_with("-->") {
				//We are reasonably confident that this is an HTML comment

				let contents = &html["<!--".len()..];
				let contents = &contents[..contents.len() - "-->".len()];

				if let Some(colon_index) = contents.find(':') {
					let label = &contents[..colon_index];
					let trailing = contents[colon_index + 1..].trim();

					match label {
						"title" => {
							title_buffer.clear();
							title_buffer.push_str(trailing);
						}

						"description" => {
							description_buffer.clear();
							description_buffer.push_str(trailing);
						}

						"author" => {
							author_buffer.clear();
							author_buffer.push_str(trailing);
						}

						_ => {}
					}
				}
			}
		}
		event
	});

	buffers.html.clear();
	html::push_html(&mut buffers.html, parser);

	buffers.output.clear();
	buffers.output.push_str("<!DOCTYPE html>\n");
	if let Some(language) = &args.language {
		let _ = writeln!(buffers.output, r#"<html lang="{}">"#, language);
	}
	buffers.output.push_str(multiline!(
		"<head>"
		r#"<meta charset="UTF-8">"#
	));
	if !buffers.title.is_empty() {
		let _ = writeln!(buffers.output, "<title>{}</title>", buffers.title);
	}
	if let Some(favicon) = &args.favicon {
		let _ = writeln!(
			buffers.output,
			r#"<link rel="shortcut icon" type="image/png" href="{}" />"#,
			favicon
		);
	}
	if !buffers.description.is_empty() {
		let _ = write!(
			buffers.output,
			multiline!(
				r#"<meta name="description" content="{description}" />"#
				r#"<meta property="og:description" content="{description}" />"#
			),
			description = buffers.description,
		);
	}
	if !buffers.author.is_empty() {
		let _ = writeln!(
			buffers.output,
			r#"<meta name="author" content="{}" />"#,
			buffers.author
		);
	}
	if let Some(opengraph_locale) = &args.opengraph_locale {
		let _ = writeln!(
			buffers.output,
			r#"<meta property="og:locale" content="{}" />"#,
			opengraph_locale
		);
	}
	if let Some(opengraph_sitename) = &args.opengraph_sitename {
		let _ = writeln!(
			buffers.output,
			r#"<meta property="og:site_name" content="{}" />"#,
			opengraph_sitename
		);
	}
	buffers.output.push_str("</head>\n");

	buffers.output.push_str(&buffers.html);
}

fn process_file(
	path: PathBuf,
	output_path: PathBuf,
	_fragments: &Fragments,
	args: &Arguments,
	buffers: &mut Buffers,
) {
	if let Some(dir_path) = output_path.parent() {
		/*
		 * NOTE: Silently swallow failure to create output path.
		 * If the path does not exist the write will still catch
		 * the error. Otherwise if this failed for some other
		 * reason but the write can still succeed then we do not
		 * care that this failed.
		 */
		let _ = std::fs::create_dir_all(dir_path);
	}

	let is_markdown = path.extension().map(|p| p.to_str()) == Some(Some("md"));

	if !is_markdown {
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

		buffers.input.clear();
		if let Err(err) = file.read_to_string(&mut buffers.input) {
			eprintln!(
				"Error reading input markdown file '{}': {}",
				path.to_string_lossy(),
				err
			);
			std::process::exit(-1);
		}

		process_markdown(_fragments, args, buffers);

		if let Err(err) = std::fs::write(&output_path, &buffers.output) {
			eprintln!(
				"Error writing HTML to path '{}': {}",
				output_path.to_string_lossy(),
				err
			);
			std::process::exit(-1);
		}
	}
}

fn main() {
	let args = arguments::parse();

	let fragments = Fragments::retrive_or_shim(args.fragments_dir.clone());

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

	/*
	 * NOTE: Silently swallow error here because it can fail
	 * if the folder does not already exist which is fine.
	 * If there really is something wrong with the path or
	 * permissions or whatever then the actual outputting will
	 * catch that. Otherwise we are uninterested in failure
	 * here.
	 */
	let _ = std::fs::remove_dir_all(&args.output_dir);

	let mut buffers = Buffers {
		input: String::new(),
		html: String::new(),
		output: String::new(),
		title: String::new(),
		description: String::new(),
		author: String::new(),
	};

	let mut stack = vec![input_dir];
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

						if let Some(Some("md")) = output_path.extension().map(|e| e.to_str()) {
							output_path.set_extension("html");
						}

						output_path
					};

					process_file(path, output_path, &fragments, &args, &mut buffers);
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
