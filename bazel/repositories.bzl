load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
load("@bazel_tools//tools/build_defs/repo:utils.bzl", "maybe")

def repositories():
    maybe(
        http_archive,
        name = "bazel_skylib",
        sha256 = "66ffd9315665bfaafc96b52278f57c7e2dd09f5ede279ea6d39b2be471e7e3aa",
        url = "https://github.com/bazelbuild/bazel-skylib/releases/download/1.4.2/bazel-skylib-1.4.2.tar.gz",
    )

    maybe(
        http_archive,
        name = "rules_rust",
        sha256 = "50272c39f20a3a3507cb56dcb5c3b348bda697a7d868708449e2fa6fb893444c",
        url = "https://github.com/bazelbuild/rules_rust/releases/download/0.22.0/rules_rust-v0.22.0.tar.gz",
        patches = ["//bazel:rules_rust.patch"],
        patch_args = ["-p1"],
    )