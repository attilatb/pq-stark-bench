// Compiles the SP1 guest programs (all bins in ../program) to ELF images that
// the host loads with include_elf!.

fn main() {
    sp1_build::build_program("../program");
    // The accelerated program: same ML-DSA-44 guest, but sha3 patched to SP1's
    // Keccak-precompile fork.
    sp1_build::build_program("../program-accel");
}
