use std::env;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

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

    let cc = std::env::var_os("SPECTRE_API_SYS_CC")
        .map(|os_str| {
            os_str
                .into_string()
                .expect("Environment variable `SPECTRE_API_SYS_CC` must be UTF-8.")
        })
        .unwrap_or_else(|| "cc".to_string());

    // Figure out the default include directories of the provided C compiler.
    let include_dirs = {
        // This should be doable with the stdlib, but I couldn't figure out how to capture the output.
        let command = cmd!(cc, "-xc", "-E", "-v", "-");
        let reader = command.stderr_to_stdout().reader().unwrap();
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
    };

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header("wrapper.h")
        .clang_args(["-x", "c", "-std=c11"])
        .clang_args(include_dirs.iter().map(|dir| format!("-I{dir}")))
        .use_core()
        .allowlist_function("spectre_.*")
        .allowlist_type("Spectre.*")
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
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
