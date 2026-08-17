//! Rebuilds the crate when a migration is added or edited.
//!
//! `embed_migrations!` bakes the SQL into the binary at compile time, but cargo
//! does not know the macro read a directory outside the crate. Without this, a
//! new migration is silently absent from the embedded set until something else
//! happens to dirty this crate -- and the failure looks like "relation does not
//! exist" at runtime, a long way from its cause.

fn main() {
    println!("cargo:rerun-if-changed=../../../migrations");
}
