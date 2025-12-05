// build.rs - Link against system notcurses library

fn main() {
    // When using vendored bindings, we still need to link against the system library
    #[cfg(feature = "tui")]
    {
        // Try pkg-config first without version requirement
        if let Ok(lib) = pkg_config::Config::new()
            .atleast_version("3.0.0")
            .probe("notcurses")
        {
            for path in lib.link_paths {
                println!("cargo:rustc-link-search=native={}", path.display());
            }
            for lib in lib.libs {
                println!("cargo:rustc-link-lib={}", lib);
            }
        } else {
            // Fallback: assume notcurses is in standard paths
            println!("cargo:rustc-link-lib=notcurses");
            println!("cargo:rustc-link-lib=notcurses-core");
        }
    }
}
