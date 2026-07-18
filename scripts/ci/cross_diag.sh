#!/usr/bin/env bash
# Toolchain census inside rb-sys-dock containers, to derive per-target
# bindgen sysroot flags for ruby-prism-sys (#110 Phase 3).
echo "=== DIAG target env ==="
env | grep -iE "^cc|cflags|bindgen|sysroot|osxcross|^cargo_|^rust" | sort
echo "=== DIAG sdk/sysroots ==="
find /opt /usr/local /usr -maxdepth 5 \( -name "*.sdk" -o -name "sysroot" -o \( -name "*mingw*" -type d \) \) 2>/dev/null | head -20
echo "=== DIAG compilers ==="
ls /usr/bin/*-gcc /usr/bin/*-clang /opt/osxcross/target/bin 2>/dev/null | head -25
echo "=== DIAG print-sysroot ==="
for cc in /usr/bin/*-gcc; do echo "$cc => $($cc -print-sysroot 2>/dev/null)"; done
echo "=== DIAG libclang ==="
find / -maxdepth 6 -name "libclang*" 2>/dev/null | head -5
echo "=== DIAG end ==="
