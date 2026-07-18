# Sourced by the cross-gem pre-script, inside rb-sys-dock containers.
#
# ruby-prism-sys runs bindgen at build time; bindgen only honors the
# dashed-target form BINDGEN_EXTRA_CLANG_ARGS_<triple-with-dashes> here
# (verified by probe: underscored and plain forms are ignored). The images
# precompute the right sysroot flags but store them under the cc-style
# underscored name, which bash also cannot re-export with dashes. Cargo's
# [env] config accepts dashed keys, so bridge the value through a generated
# .cargo/config.toml at the repo root.
# Upstream context: https://github.com/ruby/prism/issues/4175
args=$(env | sed -n 's/^BINDGEN_EXTRA_CLANG_ARGS_[a-z0-9_]*=//p' | head -n 1)
if [ -n "$args" ] && [ -n "$RUST_TARGET" ]; then
  mkdir -p .cargo
  # overwrite, not append: the pre-script can run twice (bundler fallback)
  printf '[env]\n"BINDGEN_EXTRA_CLANG_ARGS_%s" = "%s"\n' "$RUST_TARGET" "$args" > .cargo/config.toml
  echo "cross_env: BINDGEN_EXTRA_CLANG_ARGS_${RUST_TARGET} = ${args}"
fi
