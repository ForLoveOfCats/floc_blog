mod arguments;

fn main() {
	let args = arguments::parse();
	println!("Done parsing arguments, got: {:#?}", args);
}
