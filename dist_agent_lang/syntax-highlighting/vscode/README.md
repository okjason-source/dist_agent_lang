# VS Code Extension for dist_agent_lang

Professional syntax highlighting and language support for dist_agent_lang (`.dal` files) in Visual Studio Code.

## ✨ Features

- 🎨 **Syntax Highlighting**: Full color coding for all dist_agent_lang keywords
- 🔍 **IntelliSense**: Auto-completion for built-in functions and keywords  
- 🎯 **Error Detection**: Real-time syntax error highlighting
- 📝 **Auto-formatting**: Smart indentation and bracket matching
- 💬 **Comment Support**: Toggle comments with Ctrl+/
- 📁 **File Association**: Automatic `.dal` file recognition

## 🚀 Installation

### Method 1: From VS Code Marketplace (Recommended)
1. Open VS Code
2. Press `Ctrl+Shift+X` (or `Cmd+Shift+X` on Mac)
3. Search for "dist_agent_lang"
4. Click "Install"

### Method 2: Manual Installation
```bash
# Clone and install locally
git clone <repo-url>
cd dist_agent_lang/syntax-highlighting/vscode
npm install
vsce package
code --install-extension dist-agent-lang-0.1.0.vsix
```

## 🎨 Syntax Highlighting Preview

```rust
// Comments are highlighted in gray
@trust("hybrid")           // Attributes in orange
@chain("ethereum")
@compile_target("blockchain")
service DeFiService {      // Keywords in blue
    balance: int = 1000;   // Numbers in red
    
    fn transfer(amount: int) -> bool {  // Function names highlighted
        let message = "Transfer initiated";  // Strings in green
        
        if (balance >= amount) {
            balance = balance - amount;
            log::info(message, { "amount": amount });
            return true;
        }
        return false;
    }
}

agent TradingBot: ai {
    capabilities: ["read", "analyze", "trade"],
    
    fn analyze_market() {
        let data = chain::get_price("ETH");
        ai::predict(data);
    }
}
```

## 🔧 Supported Language Features

### Keywords
- **Control Flow**: `if`, `else`, `while`, `for`, `return`, `break`, `continue`
- **Declarations**: `let`, `fn`, `service`, `agent`, `spawn`, `msg`, `event`
- **Async**: `async`, `await`
- **Error Handling**: `try`, `catch`, `finally`, `throw`
- **Types**: `int`, `string`, `bool`, `any`, `null`, `Result`, `Option`

### Attributes
- **Trust Models**: `@trust`, `@secure`, `@audit`
- **Blockchain**: `@chain`, `@compile_target`
- **Performance**: `@limit`, `@cached`, `@persistent`
- **AI**: `@ai`, `@interface`

### Built-in Functions
- **Logging**: `print`, `log::info`, `log::error`
- **Math**: `add`, `multiply`, `subtract`, `divide`
- **Blockchain**: `chain::deploy`, `chain::call`, `chain::transfer`
- **AI**: `ai::predict`, `ai::analyze`, `ai::train`

## ⚙️ Configuration

### Custom Color Theme
Add to your VS Code `settings.json`:

```json
{
  "editor.tokenColorCustomizations": {
    "textMateRules": [
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
          "foreground": "#79c0ff"
        }
      }
    ]
  }
}
```

### File Association
Automatically handled, but you can manually set:
```json
{
  "files.associations": {
    "*.dal": "dist_agent_lang"
  }
}
```

## 🐛 Troubleshooting

### .dal Files Not Highlighted?
1. Check if extension is installed and enabled
2. Right-click file → "Change Language Mode" → "dist_agent_lang"
3. Restart VS Code

### Missing IntelliSense?
1. Ensure the Language Server is running
2. Check VS Code output panel for errors
3. Try reloading the window (`Ctrl+Shift+P` → "Reload Window")

## 📝 Contributing

Help improve the extension:

1. **Report Issues**: File bugs on GitHub
2. **Add Keywords**: Update `tmLanguage.json` for new language features
3. **Improve Snippets**: Add code templates for common patterns
4. **Test**: Try the extension with various `.dal` files

### Development Setup
```bash
cd syntax-highlighting/vscode
npm install
npm run compile
code .
# Press F5 to launch Extension Development Host
```

## 📄 License

MIT License - see LICENSE file for details.

## 🤝 Support

- **GitHub Issues**: Report bugs and feature requests
- **Documentation**: Full language guide in `/docs`
- **Examples**: Sample `.dal` files in `/examples`

---

**Happy coding with dist_agent_lang! 🚀**
