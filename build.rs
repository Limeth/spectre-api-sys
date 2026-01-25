use std::env;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

use bindgen::callbacks::{EnumVariantValue, ParseCallbacks};
use duct::cmd;

// macro_rules! debug {
//     ($($tokens: tt)*) => {
//         println!("cargo::warning={}", format!($($tokens)*))
//     }
// }

fn main() {
    // // Tell cargo to look for shared libraries in the specified directory
    // println!("cargo:rustc-link-search=/path/to/lib");

    // // Tell cargo to tell rustc to link the system bzip2
    // // shared library.
    // println!("cargo:rustc-link-lib=bz2");

    // Figure out the default include directories of the provided C compiler.
    let include_dirs = env::var("EXPLICITLY_INCLUDE_DEFAULT_DIRS").is_ok().then(|| {
        let compiler = cc::Build::new().get_compiler();
        let compiler_path = if compiler.path().is_absolute() {
            compiler.path().display().to_string()
        } else {
            compiler.path().file_name().unwrap().display().to_string()
        };

        // This should be doable with the stdlib, but I couldn't figure out how to capture the output.
        let command = cmd!(&compiler_path, "-xc", "-E", "-v", "-");
        let reader = command.stderr_to_stdout().reader().unwrap_or_else(|error| {
            panic!(
                "Failed to find default include directories using the compiler {compiler_path:?}. {error}",
            )
        });
        let mut collecting = false;
        let mut include_dirs = Vec::new();

        for line in BufReader::new(reader).lines() {
            let line = line.unwrap();
            let line_trimmed = line.trim().to_string();

            if line_trimmed == "End of search list." {
                break;
            }

            if collecting {
                include_dirs.push(line_trimmed.clone());
            }

            if line_trimmed == "#include <...> search starts here:" {
                collecting = true;
            }
        }

        include_dirs
    });

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let mut bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header("wrapper.h")
        .clang_args(["-x", "c", "-std=c11"])
        .use_core()
        .allowlist_function("spectre_.*")
        .allowlist_type("Spectre.*")
        .parse_callbacks(Box::new(StripEnumPrefix))
        .rustified_non_exhaustive_enum("SpectreAlgorithm.*")
        .rustified_non_exhaustive_enum("SpectreKeyPurpose.*")
        .bitfield_enum("SpectreResultClass.*")
        .bitfield_enum("SpectreResultFeature.*")
        .rustified_non_exhaustive_enum("SpectreResultType.*")
        .constified_enum_module("SpectreCounter.*")
        .rustified_non_exhaustive_enum("SpectreIdenticonColor.*")
        .rustified_non_exhaustive_enum("SpectreLogLevel.*")
        .rustified_non_exhaustive_enum("SpectreFormat.*")
        .rustified_non_exhaustive_enum("SpectreMarshalErrorType.*")
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()));

    if let Some(include_dirs) = include_dirs {
        bindings = bindings.clang_args(include_dirs.iter().map(|dir| format!("-I{dir}")));
    }

    let bindings = bindings
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}

#[derive(Debug)]
struct StripEnumPrefix;

impl ParseCallbacks for StripEnumPrefix {
    fn enum_variant_name(
        &self,
        enum_name: Option<&str>,
        original_variant_name: &str,
        _variant_value: EnumVariantValue,
    ) -> Option<String> {
        if let Some(enum_name) = enum_name
            && original_variant_name.starts_with(enum_name)
        {
            Some(original_variant_name[enum_name.len()..].to_string())
        } else {
            None
        }
    }
}
