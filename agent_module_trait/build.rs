use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn add_hash(hash: &mut u64, bytes: &[u8]) {
    for byte in bytes {
        *hash ^= u64::from(*byte);
        *hash = hash.wrapping_mul(0x100000001b3);
    }
}

fn main() {
    let rustc = env::var_os("RUSTC").expect("RUSTC is set by Cargo");
    let compiler = Command::new(rustc)
        .arg("-vV")
        .output()
        .expect("query rustc build identity");
    assert!(compiler.status.success(), "rustc -vV failed");

    let target = env::var("TARGET").expect("TARGET is set by Cargo");
    let mut hash = 0xcbf29ce484222325u64;
    for part in [
        compiler.stdout.as_slice(),
        target.as_bytes(),
        env!("CARGO_PKG_VERSION").as_bytes(),
        b"module-abi-2",
    ] {
        add_hash(&mut hash, part);
    }

    let manifest =
        PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("Cargo manifest directory"));
    let mut sources = fs::read_dir(manifest.join("src"))
        .expect("read SDK sources")
        .map(|entry| entry.expect("read SDK source entry").path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "rs"))
        .collect::<Vec<_>>();
    sources.extend([manifest.join("Cargo.toml"), manifest.join("build.rs")]);
    sources.sort();
    for source in sources {
        println!("cargo:rerun-if-changed={}", source.display());
        add_hash(
            &mut hash,
            source.file_name().unwrap().to_string_lossy().as_bytes(),
        );
        add_hash(&mut hash, &fs::read(&source).expect("read SDK source"));
    }
    println!("cargo:rustc-env=ACC_MODULE_BUILD_ID={hash}");
}
