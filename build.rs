fn main() {
	#[cfg(target_os = "windows")]
	println!("cargo:rustc-link-arg=resources.res"); // nicely, automatically compile the icon into the program
}