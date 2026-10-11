#!/usr/bin/env bash
#
# fetch-binaries.sh — download static FFmpeg/FFprobe builds for the current
# platform and place them where Tauri's `externalBin` (sidecar) expects them:
#
#   src-tauri/binaries/ffmpeg-<target-triple>[.exe]
#   src-tauri/binaries/ffprobe-<target-triple>[.exe]
#
# Sources:
#   Linux        → BtbN GPL builds (GitHub, includes NVENC/QSV/VAAPI)
#   macOS arm64  → osxexperts.net Apple Silicon static build (VideoToolbox)
#   macOS x86_64 → osxexperts.net Intel static build, fallback evermeet.cx
#
# Usage:  bash scripts/fetch-binaries.sh [target-triple]
#         DROPSHRINK_TRIPLE=aarch64-apple-darwin bash scripts/fetch-binaries.sh
#
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BIN_DIR="$REPO_ROOT/src-tauri/binaries"
mkdir -p "$BIN_DIR"

TRIPLE="${1:-${DROPSHRINK_TRIPLE:-}}"

if [[ -z "$TRIPLE" ]]; then
  OS="$(uname -s)"
  ARCH="$(uname -m)"
  case "$OS/$ARCH" in
    Linux/x86_64)  TRIPLE="x86_64-unknown-linux-gnu" ;;
    Linux/aarch64) TRIPLE="aarch64-unknown-linux-gnu" ;;
    Darwin/arm64)  TRIPLE="aarch64-apple-darwin" ;;
    Darwin/x86_64) TRIPLE="x86_64-apple-darwin" ;;
    *) echo "Unsupported platform: $OS/$ARCH — pass a target triple as arg 1" >&2; exit 1 ;;
  esac
fi

FF_OUT="$BIN_DIR/ffmpeg-$TRIPLE"
FP_OUT="$BIN_DIR/ffprobe-$TRIPLE"

if [[ -x "$FF_OUT" && -x "$FP_OUT" ]]; then
  echo "✓ Sidecar binaries already present: $BIN_DIR"
  exit 0
fi

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

fetch() { curl -fL --retry 3 --retry-delay 3 --connect-timeout 20 -o "$2" "$1"; }

install_pair() { # <ffmpeg-file> <ffprobe-file>
  cp "$1" "$FF_OUT"
  cp "$2" "$FP_OUT"
  chmod +x "$FF_OUT" "$FP_OUT"
  echo "✓ Installed: $FF_OUT"
  echo "✓ Installed: $FP_OUT"
}

case "$TRIPLE" in
  *linux*)
    echo "→ Downloading BtbN static FFmpeg for Linux…"
    fetch "https://github.com/BtbN/FFmpeg-Builds/releases/download/latest/ffmpeg-master-latest-linux64-gpl.tar.xz" "$TMP/ff.tar.xz"
    tar -xJf "$TMP/ff.tar.xz" -C "$TMP"
    install_pair "$TMP"/ffmpeg-master-latest-linux64-gpl/bin/ffmpeg "$TMP"/ffmpeg-master-latest-linux64-gpl/bin/ffprobe
    ;;

  *apple-darwin*)
    ARCH_PART="$( [[ "$TRIPLE" == aarch64* ]] && echo arm || echo intel )"
    echo "→ Resolving osxexperts.net static builds ($ARCH_PART)…"
    PAGE="$(curl -fsSL --connect-timeout 20 -A 'Mozilla/5.0' https://www.osxexperts.net/ || true)"
    FF_ZIP="$(echo "$PAGE" | grep -Eo "ffmpeg[0-9.]+${ARCH_PART}\.zip" | head -n1 || true)"
    FP_ZIP="$(echo "$PAGE" | grep -Eo "ffprobe[0-9.]+${ARCH_PART}\.zip" | head -n1 || true)"

    if [[ -n "$FF_ZIP" && -n "$FP_ZIP" ]]; then
      fetch "https://www.osxexperts.net/$FF_ZIP" "$TMP/ff.zip"
      fetch "https://www.osxexperts.net/$FP_ZIP" "$TMP/fp.zip"
      unzip -oq "$TMP/ff.zip" -d "$TMP/ff" && rm -rf "$TMP/ff/__MACOSX"
      unzip -oq "$TMP/fp.zip" -d "$TMP/fp" && rm -rf "$TMP/fp/__MACOSX"
      install_pair "$(find "$TMP/ff" -name ffmpeg -type f | head -n1)" "$(find "$TMP/fp" -name ffprobe -type f | head -n1)"
    else
      echo "  osxexperts.net unavailable — falling back to evermeet.cx (x86_64, runs via Rosetta on Apple Silicon)…"
      fetch "https://evermeet.cx/ffmpeg/getrelease/zip" "$TMP/ff.zip"
      fetch "https://evermeet.cx/ffmpeg/getrelease/ffprobe/zip" "$TMP/fp.zip"
      unzip -oq "$TMP/ff.zip" -d "$TMP/ff"
      unzip -oq "$TMP/fp.zip" -d "$TMP/fp"
      install_pair "$TMP/ff/ffmpeg" "$TMP/fp/ffprobe"
    fi
    ;;

  *)
    echo "Unsupported triple: $TRIPLE" >&2
    exit 1
    ;;
esac

echo "Done."
