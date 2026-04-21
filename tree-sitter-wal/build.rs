fn main() {
    cc::Build::new()
        .file("src/parser.c")
        .compile("tree_sitter_wal");
}
