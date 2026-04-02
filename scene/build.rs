fn main() {
    println!("cargo:rustc-link-search=/opt/oidn/lib");
    println!("cargo:rustc-link-search=native=/usr/lib64");
    println!("cargo:rustc-link-arg=-Wl,-rpath,/usr/lib64");
    println!("cargo:rustc-link-lib=OpenImageDenoise");
}
