#[cfg(not(feature = "dynamic"))]
pub mod static_link;

pub mod triple;

fn main() -> std::io::Result<()> {
    #[cfg(feature = "dynamic")]
    link_symcrypt_dynamically()?;

    #[cfg(not(feature = "dynamic"))]
    static_link::compile_and_link_symcrypt()?;

    Ok(())
}

/// Looks up an environment variable, preferring the target-specific form
/// (e.g. `X86_64_UNKNOWN_LINUX_GNU_SYMCRYPT_LIB_PATH`) over the generic
/// form (e.g. `SYMCRYPT_LIB_PATH`).
///
/// This mirrors the convention used by the `openssl-sys` crate, which
/// allows cross-compilation setups to specify different library locations
/// per target.
///
/// Also emits `cargo:rerun-if-env-changed` for each variable that is
/// consulted so Cargo re-runs the build script when they change.
#[cfg(feature = "dynamic")]
fn env_var_for_target(name: &str) -> Option<String> {
    let target = triple::Triple::get_target_triple().to_triple();
    let target_upper = target.to_uppercase().replace('-', "_");
    let prefixed = format!("{target_upper}_{name}");

    println!("cargo:rerun-if-env-changed={prefixed}");
    println!("cargo:rerun-if-env-changed={name}");

    std::env::var(&prefixed)
        .or_else(|_| std::env::var(name))
        .ok()
}

#[cfg(feature = "dynamic")]
fn link_symcrypt_dynamically() -> std::io::Result<()> {
    #[cfg(target_os = "windows")]
    {
        // Look for the .lib file during link time. We are searching the PATH for symcrypt.dll
        let lib_path = env_var_for_target("SYMCRYPT_LIB_PATH")
            .unwrap_or_else(|| panic!("SYMCRYPT_LIB_PATH environment variable not set, for more information please see: https://github.com/microsoft/rust-symcrypt/tree/main/rust-symcrypt#quick-start-guide"));
        println!("cargo:rustc-link-search=native={}", lib_path);

        println!("cargo:rustc-link-lib=dylib=symcrypt");

        // During run time, the OS will handle finding the symcrypt.dll file. The places Windows will look will be:
        // 1. The folder from which the application loaded.
        // 2. The system folder. Use the GetSystemDirectory function to retrieve the path of this folder.
        // 3. The Windows folder. Use the GetWindowsDirectory function to get the path of this folder.
        // 4. The current folder.
        // 5. The directories that are listed in the PATH environment variable.

        // For more info please see: https://learn.microsoft.com/en-us/windows/win32/dlls/dynamic-link-library-search-order
    }

    #[cfg(target_os = "linux")]
    {
        // Optionally allow callers to override where the linker/loader looks for
        // libsymcrypt.so by setting SYMCRYPT_LIB_PATH (or the target-specific
        // variant, e.g. X86_64_UNKNOWN_LINUX_GNU_SYMCRYPT_LIB_PATH). This is
        // useful when libsymcrypt.so is not installed in a default library
        // search path, or when cross-compiling.
        if let Some(lib_path) = env_var_for_target("SYMCRYPT_LIB_PATH") {
            println!("cargo:rustc-link-search=native={}", lib_path);
            // Embed an rpath so the produced binary can locate libsymcrypt.so
            // at runtime without requiring LD_LIBRARY_PATH to be set.
            println!("cargo:rustc-link-arg=-Wl,-rpath,{}", lib_path);
        }

        println!("cargo:rustc-link-lib=dylib=symcrypt"); // the "lib" prefix for libsymcrypt is implied on Linux

        // If SYMCRYPT_LIB_PATH is not set, the system linker/loader is
        // responsible for finding libsymcrypt.so.

        // If you are using AL3, you can get the required symcrypt.so via tdnf
        // If you are using Ubuntu, you can get the required symcrypt.so via PMC. Please see the quick start guide for more information.

        // If you are using a different Linux distro, you will need to configure your distro's LD linker to find the required symcrypt.so files.
        // As an example, on Ubuntu you can place your symcrypt.so files in your usr/lib/x86_64-linux-gnu/ path.
    }

    Ok(())
}
