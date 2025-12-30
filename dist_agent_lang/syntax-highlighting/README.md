# dist_agent_lang Syntax Highlighting

This directory contains syntax highlighting and language support files for various editors and IDEs.

## 🎨 **What is Syntax Highlighting?**

Syntax highlighting (also called "color coding") makes your code easier to read by:
- **Keywords** in blue/purple (`fn`, `service`, `agent`, `let`)
- **Strings** in green (`"Hello World"`)
- **Comments** in gray (`// This is a comment`)
- **Attributes** in orange (`@trust`, `@chain`, `@secure`)
- **Numbers** in red (`42`, `3.14`)
- **Operators** in different colors (`=`, `+`, `->`)

## 📁 **Available Editor Support**

### ✅ VS Code (Recommended)
Complete language support with IntelliSense and error detection.

**Installation:**
```bash
# Method 1: Install from VS Code Extensions
# 1. Open VS Code
# 2. Go to Extensions (Ctrl+Shift+X)
# 3. Search for "dist_agent_lang"
# 4. Install the extension

# Method 2: Manual Installation
cd syntax-highlighting/vscode
npm install
code --install-extension dist-agent-lang-0.1.0.vsix
```

**Features:**
- ✅ Syntax highlighting for `.dal` files
- ✅ Auto-completion for keywords
- ✅ Bracket matching and auto-closing
- ✅ Comment toggling (Ctrl+/)
- ✅ Code folding
- ✅ Error detection

### 🔧 Other Editors

#### Sublime Text
```bash
# Copy the syntax file to Sublime Text packages
cp sublime/dist_agent_lang.sublime-syntax ~/.config/sublime-text-3/Packages/User/
```

#### Vim/Neovim
```bash
# Add to your vim configuration
mkdir -p ~/.vim/syntax
cp vim/dist_agent_lang.vim ~/.vim/syntax/
echo "au BufRead,BufNewFile *.dal set filetype=dist_agent_lang" >> ~/.vimrc
```

#### Atom
```bash
# Install the language package
apm install language-dist-agent-lang
```

## 🎯 **Language Features Highlighted**

### **Keywords** (Blue/Purple)
```rust
fn main() {
    let x = 42;
    service MyService {
        // ...
    }
    agent MyAgent {
        // ...
    }
}
```

### **Attributes** (Orange)
```rust
@trust("hybrid")
@chain("ethereum")
@compile_target("blockchain")
service DeFiService {
    // ...
}
```

### **Strings & Comments** (Green/Gray)
```rust
// This is a comment
let message = "Hello from dist_agent_lang!";
/* Block comment */
```

### **Numbers & Operators** (Red/Various)
```rust
let balance = 1000;
let rate = 3.14;
let result = balance + rate;
```

### **Built-in Functions** (Cyan)
```rust
print("Hello World");
log::info("Application started");
chain::deploy("ethereum", contract);
```

## 📦 **Creating Custom Themes**

### VS Code Theme Example
```json
{
  "name": "dist_agent_lang Dark",
  "colors": {
    "editor.background": "#1e1e1e",
    "editor.foreground": "#d4d4d4"
  },
  "tokenColors": [
    {
      "scope": "entity.name.tag.attribute.dist_agent_lang",
      "settings": {
        "foreground": "#ff7b72",
        "fontStyle": "bold"
      }
    },
    {
      "scope": "keyword.declaration.dist_agent_lang",
      "settings": {
        "foreground": "#79c0ff",
        "fontStyle": "bold"
      }
    }
  ]
}
```

## 🚀 **Publishing Extensions**

### VS Code Marketplace
```bash
# Install vsce (VS Code Extension Manager)
npm install -g vsce

# Package the extension
cd syntax-highlighting/vscode
vsce package

# Publish to marketplace
vsce publish
```

### Other Marketplaces
- **Sublime Text**: Package Control
- **Vim**: vim.org, GitHub
- **Atom**: atom.io packages
- **IntelliJ**: JetBrains Plugin Repository

## 🎨 **Color Scheme Examples**

### Dark Theme
- **Background**: `#1e1e1e`
- **Keywords**: `#569cd6` (blue)
- **Attributes**: `#ff7b72` (coral)
- **Strings**: `#ce9178` (orange)
- **Comments**: `#6a9955` (green)
- **Numbers**: `#b5cea8` (light green)

### Light Theme
- **Background**: `#ffffff`
- **Keywords**: `#0000ff` (blue)
- **Attributes**: `#a31515` (red)
- **Strings**: `#a31515` (red)
- **Comments**: `#008000` (green)
- **Numbers**: `#098658` (dark green)

## 🔧 **Troubleshooting**

### .dal Files Not Highlighted?
1. Check file association: `.dal` → `dist_agent_lang`
2. Restart your editor
3. Manually set language: "Change Language Mode" → "dist_agent_lang"

### Missing Keywords?
Update the grammar file (`tmLanguage.json`) with new keywords from your lexer.

### Performance Issues?
- Disable semantic highlighting for large files
- Use simple highlighting patterns for better performance

## 📝 **Contributing**

To add support for new keywords or improve highlighting:

1. Update `dist_agent_lang.tmLanguage.json`
2. Test with sample `.dal` files
3. Update the README with new features
4. Submit a pull request

**Example: Adding new keyword**
```json
{
  "name": "keyword.declaration.dist_agent_lang",
  "match": "\\b(let|fn|service|agent|spawn|msg|event|async|await|NEW_KEYWORD)\\b"
}
```

This gives you professional syntax highlighting that makes dist_agent_lang code much easier to read and write!
