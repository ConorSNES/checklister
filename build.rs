fn main() {
	println!("cargo:rustc-link-arg=resources.res"); // nicely, automatically compile the icon into the program
}