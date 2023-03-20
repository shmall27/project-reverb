use std::path::PathBuf;

fn main() {
    let out = &PathBuf::from("/opt/homebrew/lib");
    let include = &PathBuf::from("/opt/homebrew/include");

    println!("cargo:rustc-link-search={}", out.display());
    println!("cargo:include={}", include.display());
    println!("cargo:rustc-link-lib=avcodec");
    println!("cargo:rustc-link-lib=avformat");
    println!("cargo:rustc-link-lib=avutil");

    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .use_core()
        .ctypes_prefix("cty")
        .clang_arg("-I")
        .clang_arg(include.display().to_string())
        .clang_arg("-D__STDC_CONSTANT_MACROS")
        .clang_arg("-D__STDC_FORMAT_MACROS")
        .clang_arg("-D__STDC_LIMIT_MACROS")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from("./");
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}