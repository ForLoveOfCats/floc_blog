use std::env::ArgsOs;
use std::ffi::OsString;
use std::path::PathBuf;

macro_rules! mark_used {
	($used:tt) => {};
}

macro_rules! arg_parse_error {
	( $($arg:tt)* ) => {{
		eprintln!("Error parsing arguments: {}", format_args!($($arg)*));
		std::process::exit(-1);
	}};
}

fn get_next_arg(args: &mut ArgsOs) -> OsString {
	if let Some(arg) = args.next() {
		arg
	} else {
		arg_parse_error!("Expected at least one more argument");
	}
}

macro_rules! define_flags {
	(
		$app_description:literal

		$(
			activity $activity_name:ident ($activity_short_flag:literal, $activity_long_flag:literal) $activity_blurb:literal
			$({ without_arg() $activity_without_arg_block:block })?
			$({ with_arg($activity_with_arg_name:ident) $activity_with_arg_block:block })?
		,)*

		$(
			optional $optional_name:ident ($optional_short_flag:literal, $optional_long_flag:literal) $optional_blurb:literal -> $optional_return_type:ty
			$({ without_arg() $optional_without_arg_block:block })?
			$({ with_arg($optional_with_arg_name:ident) $optional_with_arg_block:block} )?
		,)*

		$(
			required $required_name:ident ($required_short_flag:literal, $required_long_flag:literal) $required_blurb:literal -> $required_return_type:ty
			$({ without_arg() $required_without_arg_block:block })?
			$({ with_arg($required_with_arg_name:ident) $required_with_arg_block:block })?
		,)*
	) => {
		#[derive(Debug, Clone)]
		pub struct Arguments {
			$(pub $optional_name: Option<$optional_return_type> ,)*
			$(pub $required_name: $required_return_type ,)*
		}

		struct FlagParser;

		impl FlagParser {
			$($( fn $activity_name() $activity_without_arg_block )?)?
			$($( fn $activity_name($activity_with_arg_name: OsString) $activity_with_arg_block )?)?

			$($( fn $optional_name() -> $optional_return_type $optional_without_arg_block )?)?
			$($( fn $optional_name($optional_with_arg_name: OsString) -> $optional_return_type $optional_with_arg_block )?)?

			$($( fn $required_name() -> $required_return_type $required_without_arg_block )?)?
			$($( fn $required_name($required_with_arg_name: OsString) -> $required_return_type $required_with_arg_block )?)?
		}

		pub fn parse() -> Arguments {
			struct ValueTracker {
				$($optional_name: Option<$optional_return_type> ,)*
				$($required_name: Option<$required_return_type> ,)*
			}

			let mut tracker = ValueTracker {
				$($optional_name: None ,)*
				$($required_name: None ,)*
			};

			let mut args = std::env::args_os();
			args.next().expect("There was no first argument to dispose of");
			while let Some(selector) = args.next() {
				match selector.to_str() {
					$(Some($activity_short_flag) | Some($activity_long_flag) => {
						(|| {
							$(
								return FlagParser::$activity_name();
								mark_used!($activity_without_arg_block);
							)?
							$(
								let next = get_next_arg(&mut args);
								return FlagParser::$activity_name(next);
								mark_used!($activity_with_arg_block);
							)?
						})();
					})*

					$(Some($optional_short_flag) | Some($optional_long_flag) => {
						tracker.$optional_name = Some((|| {
							$(
								return FlagParser::$optional_name();
								mark_used!($optional_without_arg_block);
							)?
							$(
								let next = get_next_arg(&mut args);
								return FlagParser::$optional_name(next);
								mark_used!($optional_with_arg_block);
							)?
						})());
					})*

					$(Some($required_short_flag) | Some($required_long_flag) => {
						tracker.$required_name = Some((|| {
							$(
								return FlagParser::$required_name();
								mark_used!($required_without_arg_block);
							)?
							$(
								let next = get_next_arg(&mut args);
								return FlagParser::$required_name(next);
								mark_used!($required_with_arg_block);
							)?
						})());
					})*

					_ => arg_parse_error!("Unexpected argument '{}'", selector.to_string_lossy()),
				}
			}

			$(
				let $optional_name = tracker.$optional_name;
			)*
			$(
				let $required_name = if let Some(value) = tracker.$required_name {
					value
				} else {
					arg_parse_error!("Missing required flag '{}'", $required_long_flag);
				};
			)*

			Arguments {
				$($optional_name,)*
				$($required_name,)*
			}
		}

		pub fn print_help() {
			const INDENT: &str = "    ";

			let max_width_spaces = {
				let mut max_width = 0;

				$({
					let mut width = 0;
					width += stringify!($activity_short_flag).len();
					width += stringify!($activity_long_flag).len();
					max_width = max_width.max(width);
				})*
				$({
					let mut width = 0;
					width += stringify!($optional_short_flag).len();
					width += stringify!($optional_long_flag).len();
					max_width = max_width.max(width);
				})*
				$({
					let mut width = 0;
					width += stringify!($required_short_flag).len();
					width += stringify!($required_long_flag).len();
					max_width = max_width.max(width);
				})*

				max_width += 4; //Quotes
				max_width += 2; //`, `

				String::from_utf8(vec![b' '; max_width])
					.expect("Somehow failed to build valid string from sequence of spaces")
			};

			println!($app_description);
			println!();

			println!("USAGE:");
			println!("{}floc_blog [ACTION]", INDENT);
			println!("{}floc_blog [FLAGS]", INDENT);
			println!();

			println!("ACTIONS:");
			$(
				print!("{}", INDENT);
				print!("{} {}", stringify!($activity_short_flag), stringify!($activity_long_flag));
				let len = stringify!($activity_short_flag).len() + stringify!($activity_long_flag).len() + 4 + 2;
				println!("{}{}{}", &max_width_spaces[len..], INDENT, $activity_blurb);
			)*
			println!();

			println!("FLAGS:");
			$(
				print!("{}", INDENT);
				print!("{} {}", stringify!($optional_short_flag), stringify!($optional_long_flag));
				let len = stringify!($optional_short_flag).len() + stringify!($optional_long_flag).len() + 4 + 2;
				println!("{}{}(optional) {}", &max_width_spaces[len..], INDENT, $optional_blurb);
			)*
			$(
				print!("{}", INDENT);
				print!("{} {}", stringify!($required_short_flag), stringify!($required_long_flag));
				let len = stringify!($required_short_flag).len() + stringify!($required_long_flag).len() + 4 + 2;
				println!("{}{}(required) {}", &max_width_spaces[len..], INDENT, $required_blurb);
			)*

			println!();
		}
	};
}

define_flags! {
	"floc_blog, a small bare bones static blog generator"

	activity print_help ("-h", "--help") "Print this help message" {
		without_arg() {
			print_help();
			std::process::exit(0);
		}
	},

	optional favicon ("-s", "--favicon") "Favicon image for generated pages" -> String {
		with_arg(favicon) {
			favicon.to_string_lossy().into()
		}
	},

	optional language ("-l", "--language") "Language to specify in generated output" -> String {
		with_arg(language) {
			language.to_string_lossy().into()
		}
	},

	optional opengraph_locale ("-ol", "--opengraph-locale") "Locale for in Open Graph metadata *AND* RSS feed" -> String {
		with_arg(locale) {
			locale.to_string_lossy().into()
		}
	},

	optional opengraph_site_name ("-os", "--opengraph-site-name") "Site name for in Open Graph metadata" -> String {
		with_arg(name) {
			name.to_string_lossy().into()
		}
	},

	optional fragments_dir ("-f", "--fragments") "Directory to retrieve html footer/header/ect fragments from" -> PathBuf {
		with_arg(dir) {
			dir.into()
		}
	},

	required blog_base_url ("-u", "--base-url") "Base URL for blog subfolder" -> String {
		with_arg(url) {
			url.to_string_lossy().into()
		}
	},

	required input_dir ("-i", "--input") "Input directory to scan for .md files and assets" -> PathBuf {
		with_arg(dir) {
			dir.into()
		}
	},

	required output_dir ("-o", "--output") "Directory to place output files *DESTRUCTIVE, WILL DELETE ORIGINAL FOLDER CONTENTS*" -> PathBuf {
		with_arg(dir) {
			dir.into()
		}
	},
}
