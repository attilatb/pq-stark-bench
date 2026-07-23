// Compiles the SP1 guest programs (all bins in ../program) to ELF images that
// the host loads with include_elf!.

fn main() {
    sp1_build::build_program("../program");
}
