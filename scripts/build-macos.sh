#!/usr/bin/env bash
# Builds a signed, notarized and stapled macOS .dmg of My Pec, ready to share.
# Usage: scripts/build-macos.sh <signing env file> [--universal]
#
# The env file stays outside the repository and is never printed. It exports:
#   APPLE_SIGNING_IDENTITY  "Developer ID Application: <name> (<team id>)", in the login keychain
#   and one way to reach Apple's notary service:
#   APPLE_API_ISSUER, APPLE_API_KEY, APPLE_API_KEY_PATH   (App Store Connect API key), or
#   APPLE_ID, APPLE_PASSWORD, APPLE_TEAM_ID               (Apple ID and an app-specific password)
#
# `tauri build` signs the app and the disk image and notarizes the app; this script also notarizes the
# disk image and staples it, so Gatekeeper accepts the download itself as well as the app inside it.
# --universal builds one app for Apple Silicon and Intel; it needs `rustup target add x86_64-apple-darwin`.

set -euo pipefail

fail() {
  echo "build-macos: $*" >&2
  exit 1
}

[[ "$(uname -s)" == "Darwin" ]] || fail "this builds the macOS app; run it on a Mac"
[[ $# -ge 1 && -f "$1" ]] || fail "usage: scripts/build-macos.sh <signing env file> [--universal]"
env_file="$1"
universal=false
[[ "${2:-}" == "--universal" ]] && universal=true

cd "$(dirname "$0")/.."

set -a
# shellcheck disable=SC1090
source "$env_file"
set +a

[[ -n "${APPLE_SIGNING_IDENTITY:-}" ]] || fail "APPLE_SIGNING_IDENTITY is not set in $env_file"
security find-identity -v -p codesigning | grep -qF "\"$APPLE_SIGNING_IDENTITY\"" ||
  fail "the signing identity in $env_file is not in the keychain (security find-identity -v -p codesigning)"

if [[ -n "${APPLE_API_KEY:-}" ]]; then
  [[ -n "${APPLE_API_ISSUER:-}" && -f "${APPLE_API_KEY_PATH:-}" ]] ||
    fail "APPLE_API_KEY needs APPLE_API_ISSUER and an APPLE_API_KEY_PATH that exists"
  notary_auth=(--key "$APPLE_API_KEY_PATH" --key-id "$APPLE_API_KEY" --issuer "$APPLE_API_ISSUER")
elif [[ -n "${APPLE_ID:-}" ]]; then
  [[ -n "${APPLE_PASSWORD:-}" && -n "${APPLE_TEAM_ID:-}" ]] || fail "APPLE_ID needs APPLE_PASSWORD and APPLE_TEAM_ID"
  notary_auth=(--apple-id "$APPLE_ID" --password "$APPLE_PASSWORD" --team-id "$APPLE_TEAM_ID")
else
  fail "$env_file sets neither APPLE_API_KEY nor APPLE_ID, so nothing can be notarized"
fi

# The pnpm the repository pins, whether or not pnpm is installed.
if command -v pnpm >/dev/null; then
  pnpm=(pnpm)
else
  pnpm=(npx -y "$(node -p 'require("./package.json").packageManager')")
fi

version="$(node -p 'require("./package.json").version')"
if $universal; then
  rustup target list --installed | grep -qx x86_64-apple-darwin ||
    fail "--universal needs the Intel target: rustup target add x86_64-apple-darwin"
  "${pnpm[@]}" tauri build --bundles dmg --target universal-apple-darwin
  dmg="target/universal-apple-darwin/release/bundle/dmg/My Pec_${version}_universal.dmg"
else
  "${pnpm[@]}" tauri build --bundles dmg
  dmg="target/release/bundle/dmg/My Pec_${version}_$(uname -m | sed 's/arm64/aarch64/;s/x86_64/x64/').dmg"
fi
[[ -f "$dmg" ]] || fail "the build finished but $dmg is not there"

echo "Notarizing $dmg (usually a few minutes)"
result="$(xcrun notarytool submit "$dmg" "${notary_auth[@]}" --wait --output-format json)"
status="$(plutil -extract status raw - <<<"$result" 2>/dev/null || true)"
if [[ "$status" != "Accepted" ]]; then
  id="$(plutil -extract id raw - <<<"$result" 2>/dev/null || true)"
  fail "notarization ended as '${status:-unknown}'; see why with: xcrun notarytool log ${id:-<submission id>} <same credentials>"
fi
xcrun stapler staple -q "$dmg"

spctl -a -t open --context context:primary-signature "$dmg" ||
  fail "Gatekeeper does not accept $dmg"
echo "Ready to share: $dmg"
