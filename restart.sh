#!/usr/bin/env bash
# Recompile ptw and restart the service. Host only (needs a systemd user
# session); the last line shows whether the daemon came back up.
set -euo pipefail
cd "$(dirname "$0")"
cargo install --path crates/ptw
systemctl --user restart ptw
systemctl --user status ptw --no-pager --lines=3
