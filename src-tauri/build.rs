fn main() {
    tauri_build::build();
    #[cfg(target_os = "windows")]
    if let Ok(out_dir) = std::env::var("OUT_DIR") {
        let resource_lib = std::path::Path::new(&out_dir).join("libresource.a");
        if resource_lib.exists() {
            println!("cargo:rustc-link-arg={}", resource_lib.display());
        }
    }
}
