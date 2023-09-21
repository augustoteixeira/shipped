for f in bots/*;
do echo "Building $f..."
  cargo build --target wasm32-unknown-unknown --lib --release --manifest-path ${f}/Cargo.toml;
done
