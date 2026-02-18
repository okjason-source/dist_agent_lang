// Build script to exclude dist_agent_lang example files from Rust compilation
// These .rs files in examples/ are actually dist_agent_lang source files, not Rust

fn main() {
    // Tell Cargo to invalidate the build if examples change
    // But we don't want to compile them as Rust examples
    println!("cargo:rerun-if-changed=examples");

    // Note: Cargo automatically discovers example binaries in examples/ directory.
    // The .rs files there contain dist_agent_lang syntax (@trust, service, etc.)
    // which is not valid Rust. To prevent compilation errors, we need to either:
    // 1. Rename them to .dal files (dist_agent_lang source files)
    // 2. Move them to a different directory
    // 3. Use a pre-build script to filter them out
    //
    // For now, these files will cause compilation errors if Cargo tries to compile them.
    // They should be processed by the dist_agent_lang interpreter, not the Rust compiler.
}
