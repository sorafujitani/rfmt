# Sourced by the cross-gem pre-script, inside rb-sys-dock containers.
#
# The images precompute per-target bindgen sysroot flags but store them under
# cc-style underscored names (BINDGEN_EXTRA_CLANG_ARGS_<target_with_underscores>);
# bindgen itself only reads the dashed-target form or the plain variable, so
# the flags never reach ruby-prism-sys. Copy the value to the plain name.
# Upstream context: https://github.com/ruby/prism/issues/4175
args=$(env | sed -n 's/^BINDGEN_EXTRA_CLANG_ARGS_[a-z0-9_]*=//p' | head -n 1)
if [ -n "$args" ]; then
  export BINDGEN_EXTRA_CLANG_ARGS="$args"
  echo "cross_env: BINDGEN_EXTRA_CLANG_ARGS=$BINDGEN_EXTRA_CLANG_ARGS"
fi
