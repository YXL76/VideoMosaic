fn main() {
    // println!("cargo:rerun-if-changed=build.rs");
    #[cfg(all(target_os = "windows", target_env = "msvc"))]
    {
        println!("cargo:rustc-link-lib=dylib=mfplat");
        println!("cargo:rustc-link-lib=dylib=mfuuid");
        println!("cargo:rustc-link-lib=dylib=strmiids");
    }
}
