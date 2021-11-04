use std::collections::HashMap;

pub fn format_template(template: String, values: HashMap<&str, &str>) -> String {
	let mut output = template;

	let mut index = 0;
	while index < output.len() {
		if output.as_bytes()[index] == b'$' {
			//Start of a substitution
			let start = index;
			index += 1;

			let mut end = None;
			while index < output.len() {
				if output.as_bytes()[index] == b'$' {
					//End of substitution
					end = Some(index);
					break;
				}
				index += 1;
			}

			if let Some(end) = end {
				let key = &output[start + 1..end];
				let value = match values.get(key) {
					Some(value) => value,
					None => {
						eprintln!("Error failed to template substitute for key '{}'", key);
						std::process::exit(-1);
					}
				};

				output.replace_range(start..=end, value);
				index = start + value.len();
				continue;
			}
		}

		index += 1;
	}

	output
}
