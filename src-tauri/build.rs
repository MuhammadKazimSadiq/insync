fn main() {
    println!("cargo:rerun-if-changed=../ui/dist");
    println!("cargo:rerun-if-changed=tauri.conf.json");
    println!("cargo:rerun-if-changed=capabilities");
    tauri_build::build();
}
