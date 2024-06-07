use stabby::libloading::StabbyLibrary;

fn string_from_path_buf(path: &std::path::PathBuf) -> String {
	unsafe {
		String::from_utf8_unchecked(std::ffi::OsString::from(path.clone()).as_encoded_bytes().to_vec())
	}
}

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
	fn find_config_parser<'parser_borrow, 'self_borrow: 'parser_borrow>(&'self_borrow mut self, parser_name: &str) -> Result<&'parser_borrow mut ConfigParser<'p>, String> {
		for parser in &mut self.config_parsers {
			if parser.name.eq(parser_name) {
				return Ok(parser)
			}
		}
		let mut error = String::from(parser_name);
		error.push_str(": not loaded on ");
		let lib_path = string_from_path_buf(&self.path);
		error.push_str(&lib_path);
		return Err(error)
	}
	fn add_new_config_parser<'parser_borrow, 'self_borrow: 'parser_borrow>(&'self_borrow mut self, parser: &str, description: &str) -> Result<&'parser_borrow mut ConfigParser<'p>, Box<dyn std::error::Error + Send + Sync>>
	where
		'p: 'self_borrow
	{
		unsafe {
			if let Ok(parser) = (*(self as *mut Self)).find_config_parser(parser) { //unbounded lifetime like that described below
				return Ok(parser)
			}
			let lib = std::ptr::addr_of!(self.lib); // For unbounded lifetime use latter
			self.config_parsers.push(ConfigParser {
				name: parser.to_string(),
				description: description.to_string(),
				value: (*lib).get_stabbied(parser.as_bytes())?,
				/*
				Unbounded lifetime is used and immediately bound into 'p lifetime, I think.
				Calling one of the items within the config_parsers field of a Library after the lib field of said Library has been changed is undefined behavior.
				Should probably implement drop on Library to insure the config_parsers get dropped correctly.
				*/
			});
		}
		Ok(self.config_parsers.last_mut().unwrap())
	}
}

static mut LIBRARIES: Vec<Library> = Vec::new();

fn get_lib_path(file_name: &std::ffi::OsStr) -> Result<std::path::PathBuf, String> {
	let file_path = std::path::PathBuf::from(file_name);
	if file_path.exists() {
		return Ok(file_path)
	}
	let longer_file_path = libloading::library_filename(file_name);
	let longer_file_path = std::path::PathBuf::from(&longer_file_path);
	if longer_file_path.exists() {
		return Ok(longer_file_path)
	}
	Err(format!("{:?} either does not exist or one does not have permissions to access it!", file_name))
}

fn find_library<'l>(lib_path: &std::path::PathBuf) -> Result<&'l mut Library<'static>, String> {
	unsafe {
		for lib in &mut LIBRARIES {
			if lib.path.eq(lib_path) {
				return Ok(lib)
			}
		}
	}
	let mut lib = string_from_path_buf(lib_path);
	lib.push_str(": not loaded");
	Err(lib) //may want to call escape_debug method on String immediately before calling into method
}
fn add_new_library<'l>(lib_path: std::path::PathBuf, description: &str) -> Result<&'l mut Library<'static>, libloading::Error> {
	if let Ok(lib) = find_library(&lib_path) {
		return Ok(lib)
	}
	unsafe {
		let lib = libloading::Library::new(&lib_path)?;
		LIBRARIES.push(Library {
			path: lib_path,
			description: description.to_string(),
			lib,
			config_parsers: Vec::new(),
		});
		Ok(LIBRARIES.last_mut().unwrap())
	}
}

fn find_config_parser<'p>(lib_path: &std::path::PathBuf, parser: &str) -> Result<&'p mut ConfigParser<'static>, String> {
	let lib = find_library(lib_path)?;
	lib.find_config_parser(parser)
}
fn add_new_config_parser<'p>(lib_path: &std::path::PathBuf, parser: &str, description: &str) -> Result<&'p mut ConfigParser<'static>, Box<dyn std::error::Error + Send + Sync>> {
	unsafe {
		let lib = find_library(lib_path)?;
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
