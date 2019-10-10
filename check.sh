
#!/usr/bin/env bash
set -e

PROJECT_ROOT="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null && pwd )"

export CARGO_INCREMENTAL=0

# These don't work on CircleCI unfortunately
#bold=$(tput bold)
#normal=$(tput sgr0)

# Save current directory.
pushd . >/dev/null

for SRC in runtime/wasm
do
  #echo "${bold}Checking webassembly binary in $SRC...${normal}"
  echo "Checking webassembly binary in $SRC..."
  cd "$PROJECT_ROOT/$SRC"

  ./check.sh

  cd - >> /dev/null
done

# Restore initial directory.
popd >/dev/null