

extern crate skeptic;

fn main() {
    println!("rerun-if-changed=README.md");
    println!("rerun-if-changed=src/lib.rs");
    println!("rerun-if-changed=cardparse_derive/src/lib.rs");
    // generates doc tests for `README.md`.
    skeptic::generate_doc_tests(&["README.md"]);
}