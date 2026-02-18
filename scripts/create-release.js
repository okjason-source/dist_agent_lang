#!/usr/bin/env node

const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');

const VERSION = process.env.VERSION || '1.0.2';
const RELEASE_DIR = `dist_agent_lang-${VERSION}`;
const PLATFORMS = ['x86_64-unknown-linux-gnu', 'x86_64-apple-darwin', 'x86_64-pc-windows-msvc'];

console.log(`🚀 Creating release package for dist_agent_lang v${VERSION}`);

// Create release directory
if (fs.existsSync(RELEASE_DIR)) {
    fs.rmSync(RELEASE_DIR, { recursive: true });
}
fs.mkdirSync(RELEASE_DIR);

// Copy only essential files (minimal package)
console.log('📁 Copying essential files only...');

// Copy binary
const binDir = path.join(RELEASE_DIR, 'bin');
fs.mkdirSync(binDir, { recursive: true });
const binPath = path.join(__dirname, '..', 'target', 'release', 'dist_agent_lang');
if (fs.existsSync(binPath)) {
    fs.copyFileSync(binPath, path.join(binDir, 'dist_agent_lang'));
    fs.chmodSync(path.join(binDir, 'dist_agent_lang'), '755');
}

// Copy examples (.dal files only, exclude tests)
const examplesDir = path.join(RELEASE_DIR, 'examples');
fs.mkdirSync(examplesDir, { recursive: true });
const examplesSrc = path.join(__dirname, '..', 'examples');
if (fs.existsSync(examplesSrc)) {
    const files = fs.readdirSync(examplesSrc);
    files.forEach(file => {
        // Only include .dal files, exclude test files and .bak files
        if (file.endsWith('.dal') && !file.includes('test_') && !file.includes('debug')) {
            fs.copyFileSync(
                path.join(examplesSrc, file),
                path.join(examplesDir, file)
            );
        }
    });
}

// Copy essential documentation
const essentialFiles = ['README.md', 'LICENSE', 'CHANGELOG.md'];
essentialFiles.forEach(file => {
    const src = path.join(__dirname, '..', file);
    const dest = path.join(RELEASE_DIR, file);
    if (fs.existsSync(src)) {
        fs.copyFileSync(src, dest);
    }
});

// Copy installation script
const installScriptSrc = path.join(__dirname, '..', 'scripts', 'install.sh');
if (fs.existsSync(installScriptSrc)) {
    fs.copyFileSync(installScriptSrc, path.join(RELEASE_DIR, 'install.sh'));
    fs.chmodSync(path.join(RELEASE_DIR, 'install.sh'), '755');
}

// Create platform-specific builds
console.log('🔨 Building for different platforms...');
PLATFORMS.forEach(platform => {
    try {
        console.log(`Building for ${platform}...`);
        execSync(`cargo build --release --target ${platform}`, { 
            stdio: 'inherit',
            cwd: path.join(__dirname, '..')
        });
        
        const binDir = path.join(RELEASE_DIR, 'bin', platform);
        fs.mkdirSync(binDir, { recursive: true });
        
        const binName = platform.includes('windows') ? 'dist_agent_lang.exe' : 'dist_agent_lang';
        const binPath = path.join(__dirname, '..', 'target', platform, 'release', binName);
        
        if (fs.existsSync(binPath)) {
            fs.copyFileSync(binPath, path.join(binDir, binName));
            console.log(`✅ Built ${platform}`);
        }
    } catch (error) {
        console.log(`❌ Failed to build for ${platform}: ${error.message}`);
    }
});

// Create installation script
const installScript = `#!/bin/bash
# dist_agent_lang Installation Script v${VERSION}

echo "🚀 Installing dist_agent_lang v${VERSION}..."

# Detect platform
PLATFORM=""
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    PLATFORM="x86_64-unknown-linux-gnu"
elif [[ "$OSTYPE" == "darwin"* ]]; then
    PLATFORM="x86_64-apple-darwin"
elif [[ "$OSTYPE" == "msys" ]] || [[ "$OSTYPE" == "cygwin" ]]; then
    PLATFORM="x86_64-pc-windows-msvc"
else
    echo "❌ Unsupported platform: $OSTYPE"
    exit 1
fi

# Install binary
BIN_NAME="dist_agent_lang"
if [[ "$PLATFORM" == "x86_64-pc-windows-msvc" ]]; then
    BIN_NAME="dist_agent_lang.exe"
fi

INSTALL_DIR="/usr/local/bin"
if [[ ! -w "$INSTALL_DIR" ]]; then
    INSTALL_DIR="$HOME/.local/bin"
    mkdir -p "$INSTALL_DIR"
fi

cp "bin/$PLATFORM/$BIN_NAME" "$INSTALL_DIR/$BIN_NAME"
chmod +x "$INSTALL_DIR/$BIN_NAME"

echo "✅ dist_agent_lang installed to $INSTALL_DIR/$BIN_NAME"
echo "🎉 Installation complete! Run '$BIN_NAME --help' to get started."
`;

fs.writeFileSync(path.join(RELEASE_DIR, 'install.sh'), installScript);
fs.chmodSync(path.join(RELEASE_DIR, 'install.sh'), '755');

// Create Windows installation script
const installScriptWin = `@echo off
REM dist_agent_lang Installation Script v${VERSION}

echo 🚀 Installing dist_agent_lang v${VERSION}...

set PLATFORM=x86_64-pc-windows-msvc
set BIN_NAME=dist_agent_lang.exe
set INSTALL_DIR=%USERPROFILE%\\AppData\\Local\\dist_agent_lang

if not exist "%INSTALL_DIR%" mkdir "%INSTALL_DIR%"

copy "bin\\%PLATFORM%\\%BIN_NAME%" "%INSTALL_DIR%\\%BIN_NAME%"

echo ✅ dist_agent_lang installed to %INSTALL_DIR%\\%BIN_NAME%
echo 🎉 Installation complete! Add %INSTALL_DIR% to your PATH and run '%BIN_NAME% --help' to get started.
pause
`;

fs.writeFileSync(path.join(RELEASE_DIR, 'install.bat'), installScriptWin);

// Create package manifest
const manifest = {
    version: VERSION,
    release_date: new Date().toISOString(),
    platforms: PLATFORMS,
    features: [
        "AI Agent Framework",
        "Blockchain Integration", 
        "Cross-Chain Operations",
        "Smart Contract Development",
        "Oracle Integration",
        "KYC/AML Compliance",
        "Multi-Target Compilation",
        "Trust Model System",
        "Interface Generation"
    ],
    examples: fs.existsSync(path.join(RELEASE_DIR, 'examples')) 
        ? fs.readdirSync(path.join(RELEASE_DIR, 'examples')).filter(f => f.endsWith('.dal'))
        : []
};

fs.writeFileSync(path.join(RELEASE_DIR, 'manifest.json'), JSON.stringify(manifest, null, 2));

// Create archive
console.log('📦 Creating release archive...');
try {
    execSync(`tar -czf ${RELEASE_DIR}.tar.gz ${RELEASE_DIR}`, { stdio: 'inherit' });
    console.log(`✅ Created ${RELEASE_DIR}.tar.gz`);
} catch (error) {
    console.log(`❌ Failed to create tar.gz: ${error.message}`);
}

try {
    execSync(`zip -r ${RELEASE_DIR}.zip ${RELEASE_DIR}`, { stdio: 'inherit' });
    console.log(`✅ Created ${RELEASE_DIR}.zip`);
} catch (error) {
    console.log(`❌ Failed to create zip: ${error.message}`);
}

console.log(`🎉 Release package created: ${RELEASE_DIR}`);
console.log(`📦 Archives: ${RELEASE_DIR}.tar.gz, ${RELEASE_DIR}.zip`);
