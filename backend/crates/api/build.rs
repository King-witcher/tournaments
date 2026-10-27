// `sqlx::migrate!` embute as migrations na compilação; isto recompila quando
// um arquivo de migration muda.
fn main() {
    println!("cargo:rerun-if-changed=../../migrations");
}
