// Compiles the guest crate to a RISC Zero ELF and generates constants
// (the ELF bytes and the image ID) that the host links against.

fn main() {
    risc0_build::embed_methods();
}
