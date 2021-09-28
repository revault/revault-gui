set -ex

rustup component add llvm-tools-preview
if ! command -v grcov &>/dev/null; then
    cargo +nightly install grcov
fi

cargo clean

rm -f revault_gui_coverage_*.profraw
LLVM_PROFILE_FILE="revault_gui_coverage_%m.profraw" RUSTFLAGS="-Zinstrument-coverage" RUSTDOCFLAGS="$RUSTFLAGS -Z unstable-options --persist-doctests target/debug/doctestbins" cargo +nightly build --all-features
LLVM_PROFILE_FILE="revault_gui_coverage_%m.profraw" RUSTFLAGS="-Zinstrument-coverage" RUSTDOCFLAGS="$RUSTFLAGS -Z unstable-options --persist-doctests target/debug/doctestbins" cargo +nightly test --all-features

grcov . --binary-path ./target/debug/ -t html --branch --ignore-not-existing --llvm -o ./target/grcov/
firefox target/grcov/index.html

set +ex
