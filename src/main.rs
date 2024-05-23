use stabby::libloading::StabbyLibrary;

struct ConfigParser<'p> {
	name: String,
	description: String,
	value: stabby::libloading::Symbol<'p, extern "C" fn(stabby::string::String) -> stabby::result::Result<stabby::vec::Vec<snippet_config_types::Scope>, stabby::string::String>>,
}
impl<'p> ConfigParser<'p> {
	fn parse_file(&'p self, file: &str) -> Result<stabby::vec::Vec<snippet_config_types::Scope>, stabby::string::String> {
		let parser = &self.value;
		parser(stabby::string::String::from(file)).into()
	}
}

struct Library<'p> {
	path: std::path::PathBuf,
	description: String,
	lib: libloading::Library,
	config_parsers: Vec<ConfigParser<'p>>,
}
impl<'p> Library<'p> {
	fn add_new_config_parser(&'p mut self, parser: &str, description: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
		if let Some(_) = self.find_config_parser(parser) {
			return Ok(())
		}
		unsafe {
			let value = self.lib.get_stabbied(parser.as_bytes())?;
			self.config_parsers.push(ConfigParser {
				name: parser.to_string(),
				description: description.to_string(),
				value,
			});
			Ok(())
		}
	}
	fn find_config_parser(&self, parser: &str) -> Option<usize> {
		let mut i: usize = 0;
		let parsers = &self.config_parsers;
		let len = parsers.len();
		while i < len {
			if parsers[i].name.eq(parser) {
				return Some(i)
			}
			i += 1;
		}
		None
	}
}

static mut LIBRARIES: Vec<Library> = Vec::new();
static mut SCOPES: Vec<snippet_config_types::Scope> = Vec::new();

fn get_lib_path(file_name: &std::ffi::OsStr) -> Option<std::path::PathBuf> {
	let file_path = std::path::PathBuf::from(file_name);
	if file_path.exists() {
		return Some(file_path)
	}
	let longer_file_path = libloading::library_filename(file_name);
	let longer_file_path = std::path::PathBuf::from(&longer_file_path);
	if longer_file_path.exists() {
		return Some(longer_file_path)
	}
	eprintln!("{:?} either does not exist or one does not have permissions to access it!", file_name);
	None
}

fn find_library(lib: &std::path::PathBuf) -> Option<usize> {
	unsafe {
		let mut i: usize = 0;
		let len = LIBRARIES.len();
		while i < len {
			let path = &LIBRARIES[i].path;
			if path == lib {
				return Some(i)
			}
			i += 1;
		}
	}
	None
}

fn add_new_library(lib_path: std::path::PathBuf, description: &str) -> Result<(), libloading::Error> {
	if let Some(_) = find_library(&lib_path) {
		return Ok(())
	}
	unsafe {
		let lib = libloading::Library::new(&lib_path)?;
		LIBRARIES.push(Library {
			path: lib_path,
			description: description.to_string(),
			lib,
			config_parsers: Vec::new(),
		});
	}
	Ok(())
}

fn find_config_parser(lib_path: &std::path::PathBuf, parser: &str) -> Option<usize> {
	unsafe {
		let mut i: usize = 0;
		let len = LIBRARIES.len();
		let lib = loop {
			if i == len {
				return None
			}
			let lib = &LIBRARIES[i];
			if lib.path.eq(lib_path) {
				break lib
			}
			i += 1;
		};
		lib.find_config_parser(parser)
	}
}

fn add_new_config_parser(lib_path: &std::path::PathBuf, parser: &str, description: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
	unsafe {
		let lib_index = if let Some(lib_index) = find_config_parser(lib_path, parser) {
			lib_index
		} else {
			let mut lib_as_os_string = std::ffi::OsString::from(lib_path.clone());
			lib_as_os_string.push(": not loaded");
			return Err(String::from_utf8_unchecked(lib_as_os_string.as_encoded_bytes().to_vec()).into()) //may want to call escape_debug method on String immediately before calling into method
		};
		let lib = &mut LIBRARIES[lib_index];
		if let Some(_) = lib.find_config_parser(parser) {
			return Ok(())
		}
		lib.add_new_config_parser(parser, description)
	}
}

fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod tests {
	use super::*;
	#[test]
	fn unknown_lib() {
		unsafe {
			if let Err(e) = libloading::Library::new("./test.so") {
				eprintln!("{:?}", e);
			}
		}
	}
	#[test]
	fn unloaded_lib() {
		let lib_path = std::path::PathBuf::from("./test.so");
		if let Err(e) = add_new_config_parser(&lib_path, "vscode_parser", "VSCode compatable parser") {
			eprintln!("{:?}", e);
		}
	}
}
