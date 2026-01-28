#!/bin/bash
# Import GitHub labels from labels.json
# Requires GitHub CLI (gh): https://cli.github.com/

set -e

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${YELLOW}Importing GitHub labels...${NC}"

# Check if gh CLI is installed
if ! command -v gh &> /dev/null; then
    echo -e "${RED}Error: GitHub CLI (gh) is not installed.${NC}"
    echo "Install it from: https://cli.github.com/"
    exit 1
fi

# Check if authenticated
if ! gh auth status &> /dev/null; then
    echo -e "${YELLOW}Not authenticated. Running gh auth login...${NC}"
    gh auth login
fi

# Get repository owner and name
REPO=$(gh repo view --json nameWithOwner -q .nameWithOwner 2>/dev/null || echo "")
if [ -z "$REPO" ]; then
    echo -e "${RED}Error: Could not determine repository.${NC}"
    echo "Make sure you're in a git repository and GitHub CLI is configured."
    exit 1
fi

echo -e "${GREEN}Repository: $REPO${NC}"
echo ""

# Function to create a label
create_label() {
    local name=$1
    local color=$2
    local description=$3
    
    echo -n "Creating label '$name'... "
    
    if gh api "repos/$REPO/labels" \
        -X POST \
        -f name="$name" \
        -f color="$color" \
        -f description="$description" &> /dev/null; then
        echo -e "${GREEN}✓${NC}"
    else
        # Try to update if it already exists
        if gh api "repos/$REPO/labels/$name" \
            -X PATCH \
            -f color="$color" \
            -f description="$description" &> /dev/null; then
            echo -e "${YELLOW}Updated${NC}"
        else
            echo -e "${RED}Failed${NC}"
        fi
    fi
}

# Import labels from labels.json structure
# Essential labels
create_label "good-first-issue" "0e8a16" "Good for newcomers"
create_label "help-wanted" "008672" "Extra attention is needed"
create_label "bug" "d73a4a" "Something isn't working"
create_label "enhancement" "a2eeef" "New feature or request"
create_label "documentation" "0075ca" "Improvements or additions to documentation"
create_label "testing" "f9d71c" "Testing related"
create_label "security" "ee0701" "Security issue"
create_label "question" "d876e3" "Further information is requested"

# Status labels
create_label "wontfix" "ffffff" "This will not be worked on"
create_label "duplicate" "cfd3d7" "This issue or pull request already exists"
create_label "invalid" "e4e669" "This doesn't seem right"
create_label "blocked" "b60205" "Blocked by another issue"

# Priority labels
create_label "priority: high" "b60205" "High priority issue"
create_label "priority: medium" "fbca04" "Medium priority issue"
create_label "priority: low" "0e8a16" "Low priority issue"

# Area labels
create_label "area: lexer" "1d76db" "Related to lexer/tokenization"
create_label "area: parser" "1d76db" "Related to parser/AST"
create_label "area: runtime" "1d76db" "Related to runtime execution"
create_label "area: stdlib" "1d76db" "Related to standard library"
create_label "area: blockchain" "1d76db" "Related to blockchain features"
create_label "area: ai" "1d76db" "Related to AI agent features"
create_label "area: security" "1d76db" "Related to security features"
create_label "area: http" "1d76db" "Related to HTTP server"
create_label "area: ffi" "1d76db" "Related to FFI (Foreign Function Interface)"

echo ""
echo -e "${GREEN}✓ Label import complete!${NC}"
echo ""
echo "You can now use these labels in your issues and pull requests."
