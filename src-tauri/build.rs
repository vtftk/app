use std::{env, path::PathBuf};

use deno_core::extension;

fn main() {
    tauri_build::build();
    create_runtime_snapshot();
}

/// Creates a snapshot of the runtime wrapper code
///
/// This makes the code startup faster and prevents needing to
/// tack on the wrapper script to every script we want to run.
///
/// Instead a snapshot of the runtime gets compiled here and
/// loaded by the runtime on startup
fn create_runtime_snapshot() {
    extension!(
        // extension name
        api_extension,
        // the entrypoint to our extension
        esm_entry_point = "vtftk:runtime",
        // list of all JS files in the extension
        esm = [
            dir "../scripting/dist",
            "vtftk:runtime" = "runtime.js"
        ]
    );

    let out_dir: PathBuf = env::var_os("OUT_DIR").expect("missing out dir").into();
    let snapshot_path = out_dir.join("SCRIPT_RUNTIME_SNAPSHOT.bin");

    let snapshot = deno_core::snapshot::create_snapshot(
        deno_core::snapshot::CreateSnapshotOptions {
            cargo_manifest_dir: env!("CARGO_MANIFEST_DIR"),
            startup_snapshot: None,
            skip_op_registration: false,
            extensions: vec![api_extension::init()],
            with_runtime_cb: None,
            extension_transpiler: None,
        },
        None,
    )
    .unwrap();

    std::fs::write(snapshot_path, snapshot.output).expect("failed to write snapshot");
}
