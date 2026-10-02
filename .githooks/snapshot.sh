# Sourced by the hooks: sync what's staged into a fixed dir in .git and cd
# there, so the checks see the commit, not the working tree, and never write
# to it. rsync rewrites only the files that changed, so cargo's fingerprints
# (which go by mtime) stay fresh and its builds incremental.
repo=$(git rev-parse --show-toplevel)
cd "$repo"
git_dir=$(git rev-parse --absolute-git-dir)
snapshot="$git_dir/pre-commit-tree"
staged=$(mktemp -d)
trap 'rm -rf "$staged"' EXIT
git checkout-index --all --prefix="$staged/"
rsync -rc --delete "$staged/" "$snapshot/"
export CARGO_TARGET_DIR="$repo/target"
cd "$snapshot"
