#!/usr/bin/env bash
set -e
echo "Installing latest dal from https://github.com/okjason-source/dist_agent_lang..."
cargo install --git https://github.com/okjason-source/dist_agent_lang.git --package dist_agent_lang --bin dal
echo "Done. Run 'dal --version' to confirm."
