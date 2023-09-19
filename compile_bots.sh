for f in bots/*;
do echo "Building $f..."
  cargo build --manifest-path ${f}/Cargo.toml;
done
