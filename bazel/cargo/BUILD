load("@rules_rust//crate_universe:defs.bzl", "crates_vendor")

exports_files([
    "Cargo.Bazel.lock",
])

crates_vendor(
    name = "crates_vendor",
    cargo_lockfile = "//bazel/cargo:Cargo.Bazel.lock",
    manifests = ["//:Cargo.toml"],
    mode = "remote",
    tags = ["manual"],
    vendor_path = "//bazel/cargo/remote",
)