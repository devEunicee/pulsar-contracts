#!/bin/bash
# SPDX-License-Identifier: MIT
#
# Check that all Rust source files have the SPDX license header.
# This script is used in CI to enforce license compliance.

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(dirname "$SCRIPT_DIR")"

# Find all .rs files and check for license header
MISSING_HEADERS=()

while IFS= read -r file; do
    # Skip test files in target directory
    if [[ "$file" == *"/target/"* ]]; then
        continue
    fi
    
    # Check if file starts with SPDX license identifier
    if ! head -1 "$file" | grep -q "SPDX-License-Identifier"; then
        MISSING_HEADERS+=("$file")
    fi
done < <(find "$REPO_ROOT" -name "*.rs" -type f)

if [ ${#MISSING_HEADERS[@]} -gt 0 ]; then
    echo "❌ License header check failed!"
    echo ""
    echo "The following files are missing SPDX license headers:"
    echo ""
    for file in "${MISSING_HEADERS[@]}"; do
        echo "  - $file"
    done
    echo ""
    echo "Add the following line to the top of each file:"
    echo "  // SPDX-License-Identifier: MIT"
    echo ""
    exit 1
fi

echo "✅ All Rust files have SPDX license headers"
exit 0
