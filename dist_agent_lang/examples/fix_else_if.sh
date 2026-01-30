#!/bin/bash
# Convert "} else if (condition) {" to "} else { if (condition) { ..." in .dal files

for file in *.dal; do
    # This sed command converts else-if to else { if
    # It handles the pattern: } else if (condition) {
    perl -i -pe 's/(\s*)\} else if \(([^)]+)\) \{/$1} else {\n$1    if ($2) {/g' "$file"
done

echo "Converted else-if to else { if } in all .dal files"
