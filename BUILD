load("@rules_rust//rust:defs.bzl", "rust_binary")
load("@header_injection//bazel/cargo/remote:defs.bzl", "aliases", "all_crate_deps")

exports_files([
    "Cargo.toml",
])

rust_binary(
    name = "header_injection",
    srcs = glob(["src/*.rs"]),
    crate_type = "cdylib",
    edition = "2021",
    out_binary = True,
    deps = all_crate_deps(
        normal = True,
    )
)
