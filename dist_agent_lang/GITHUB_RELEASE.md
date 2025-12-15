# GitHub Release Guide

## Prerequisites
- GitHub account
- GitHub CLI (`gh`) installed, OR use GitHub web interface

## Option A: Using GitHub CLI (Recommended)

### 1. Create GitHub Repository
```bash
# Create a new repository on GitHub first (via web interface)
# Then initialize and push:

cd dist_agent_lang
git init
git add .
git commit -m "Initial release v1.0.0"

# Add remote (replace with your GitHub username/repo)
git remote add origin https://github.com/YOUR_USERNAME/dist_agent_lang.git
git branch -M main
git push -u origin main
```

### 2. Create Release
```bash
# Make sure you're in the project directory
cd dist_agent_lang

# Create release with GitHub CLI
gh release create v1.0.0 \
    --title "dist_agent_lang v1.0.0" \
    --notes "$(cat CHANGELOG.md)" \
    dist_agent_lang-1.0.0.tar.gz \
    dist_agent_lang-1.0.0.zip

# Or create draft release first
gh release create v1.0.0 \
    --draft \
    --title "dist_agent_lang v1.0.0" \
    --notes "$(cat CHANGELOG.md)" \
    dist_agent_lang-1.0.0.tar.gz \
    dist_agent_lang-1.0.0.zip
```

## Option B: Using GitHub Web Interface

### 1. Push Code to GitHub
```bash
git init
git add .
git commit -m "Initial release v1.0.0"
git remote add origin https://github.com/YOUR_USERNAME/dist_agent_lang.git
git push -u origin main
```

### 2. Create Release via Web
1. Go to your repository on GitHub
2. Click "Releases" → "Create a new release"
3. Tag: `v1.0.0`
4. Title: `dist_agent_lang v1.0.0`
5. Description: Copy contents from `CHANGELOG.md`
6. Upload files:
   - `dist_agent_lang-1.0.0.tar.gz`
   - `dist_agent_lang-1.0.0.zip`
7. Click "Publish release"

## What Gets Pushed

### Required Files (for GitHub)
- Source code (`src/`)
- Documentation (`README.md`, `CHANGELOG.md`, `LICENSE`)
- Configuration (`Cargo.toml`, `package.json`)
- Examples (`examples/*.dal`)

### Optional Files
- Build scripts (`Makefile`, `scripts/`)
- Documentation (`docs/`, `INSTALLATION.md`, `USAGE.md`)

### Should NOT Push
- `target/` (build artifacts)
- `dist_agent_lang-1.0.0/` (release package directory)
- `*.tar.gz`, `*.zip` (release archives - upload via Releases instead)
- `.DS_Store` (macOS files)

## .gitignore Recommendations

Create `.gitignore`:
```
# Build artifacts
target/
dist_agent_lang-*/
*.tar.gz
*.zip

# IDE
.idea/
.vscode/
*.swp
*.swo

# OS
.DS_Store
Thumbs.db

# Logs
*.log
```

## Quick Release Checklist

- [ ] Code is committed
- [ ] Version numbers updated (Cargo.toml, package.json)
- [ ] CHANGELOG.md updated
- [ ] README.md is current
- [ ] Package created (`make package`)
- [ ] Release notes prepared
- [ ] GitHub repository created
- [ ] Code pushed to GitHub
- [ ] Release created with package files

## After Release

Users can install via:
```bash
# Download from GitHub Releases
wget https://github.com/YOUR_USERNAME/dist_agent_lang/releases/download/v1.0.0/dist_agent_lang-1.0.0.tar.gz

# Extract and install
tar -xzf dist_agent_lang-1.0.0.tar.gz
cd dist_agent_lang-1.0.0
./install.sh
```

