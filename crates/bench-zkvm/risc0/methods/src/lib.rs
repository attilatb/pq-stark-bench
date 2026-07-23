// Re-exports the generated guest artifacts (ELF images and image IDs).
// The names are derived from the guest binary names by risc0-build.
include!(concat!(env!("OUT_DIR"), "/methods.rs"));
