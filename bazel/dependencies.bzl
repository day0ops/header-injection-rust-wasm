load("@header_injection//bazel/cargo/remote:defs.bzl", "crate_repositories")
load("@rules_rust//rust:repositories.bzl", "rust_repositories")
load("@rules_rust//crate_universe:repositories.bzl", "crate_universe_dependencies")

def dependencies():
    rust_repositories()
    crate_universe_dependencies()
    crate_repositories()