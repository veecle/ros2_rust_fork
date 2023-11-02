use std::path::Path;

fn main() {
	// Special linking when building with middleware
    if let Ok(_) = std::env::var("CARGO_FEATURE_WITH_MIDDLEWARE") {
		let lib_dir = Path::new("../../../lib")
			.canonicalize()
			.expect("Could not find '../../../lib'");
		// This allows building Rust packages that depend on message crates without
		// sourcing the install directory first.
		println!("cargo:rustc-link-search={}", lib_dir.display());
    }
}
