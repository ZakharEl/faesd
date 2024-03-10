use stabby::libloading::StabbyLibrary;

struct Parser<'p> {
	name: String,
	description: String,
	value: stabby::libloading::Symbol<'p, extern "C" fn(stabby::string::String) -> snippet_config_types::Config>,
}

struct Library<'p> {
	path: std::path::PathBuf,
	description: String,
	lib: libloading::Library,
	parsers: Vec<Parser<'p>>,
}
impl<'p> Library<'p> {
	fn add_new_config_getter(&'p mut self, parser: &str, description: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
		if let Some(_) = self.find_config_getter(parser) {
			return Ok(())
		}
		unsafe {
			let value = self.lib.get_stabbied(parser.as_bytes())?;
			self.parsers.push(Parser {
				name: parser.to_string(),
				description: description.to_string(),
				value,
			});
			Ok(())
		}
	}
	fn find_config_getter(&self, parser: &str) -> Option<usize> {
		let mut i: usize = 0;
		let parsers = &self.parsers;
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
static mut PARSERS: Vec<Parser> = Vec::new();

fn find_library(lib: &std::ffi::OsStr) -> Option<usize> {
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

fn add_new_library(lib: &std::ffi::OsStr, description: &str) -> Result<(), libloading::Error> {
	if let Some(_) = find_library(lib) {
		return Ok(())
	}
	let lib_filename = libloading::library_filename(lib);
	if let Some(_) = find_library(&lib_filename) {
		return Ok(())
	}
	unsafe {
		if let Ok(lib) = libloading::Library::new(&lib_filename) {
			LIBRARIES.push(Library {
				path: std::path::PathBuf::from(&lib_filename),
				description: description.to_string(),
				lib,
				parsers: Vec::new(),
			});
			return Ok(())
		}
		let actual_lib = libloading::Library::new(lib)?;
		LIBRARIES.push(Library {
			path: std::path::PathBuf::from(lib),
			description: description.to_string(),
			lib: actual_lib,
			parsers: Vec::new(),
		});
	}
	Ok(())
}

fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod tests {
	use super::*;
	#[test]
	fn pointer_eq() {
		let t1: fn(lib: &std::ffi::OsStr) -> Option<usize> = find_library;
		let t2: fn(lib: &std::ffi::OsStr) -> Option<usize> = find_library;
		assert_eq!(t1 as usize, t2 as usize);
	}
}
