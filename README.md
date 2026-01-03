# 9paste

A **privacy-focused** clipboard transformer that automatically cleans, formats, and transforms clipboard text as you paste. Create reusable "recipes" to standardize pasting with `Ctrl+V`.

![Cross-Platform](https://img.shields.io/badge/platform-Linux%20%7C%20macOS%20%7C%20Windows-blue)
![Rust](https://img.shields.io/badge/rust-1.70%2B-orange)
![License](https://img.shields.io/badge/license-MIT-green)

## ✨ Features

### 🧹 40+ Text Transformations
- **Whitespace**: Normalize, trim, remove empty lines
- **Case**: lowercase, UPPERCASE, Title Case, camelCase, snake_case, etc.
- **Lines**: Sort, reverse, deduplicate, add/remove line numbers
- **Cleanup**: Fix smart quotes, remove emojis, strip formatting
- **Content**: Remove URLs, emails, phone numbers, markdown
- **Code**: Convert tabs/spaces, fix line endings
- **HTML**: Encode/decode entities
- **And more...**

### 📝 Reusable Recipes
Create named collections of transformations that you use frequently:
- "Plain Text" - Strip all formatting from web copies
- "Clean Code" - Fix smart quotes and normalize whitespace
- "Privacy Mode" - Remove emails and phone numbers
- Build your own!

### 🔄 Auto-Transform
Set a recipe as active and every paste is automatically transformed.

### ⌨️ Hotkey Support
- Quick toggle transformation on/off
- Open recipe quick menu
- Open dashboard
- Custom hotkeys per recipe

### 🔒 Privacy First
- **100% local processing** - Nothing leaves your device
- **No cloud** - Works offline
- **No tracking** - We can't see your clipboard data

## 🚀 Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/skorotkiewicz/9paste.git
cd 9paste

# Build release version
cargo build --release

# Install to PATH
cargo install --path .
```

### System Dependencies

**Linux (X11/Wayland):**
```bash
# Debian/Ubuntu
sudo apt install libxcb1-dev libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev xclip

# On Wayland, you may also need:
sudo apt install libxkbcommon-dev
```

**macOS:**
No additional dependencies required.

**Windows:**
No additional dependencies required.

## 📖 Usage

### Dashboard (GUI)

```bash
# Open the dashboard (default if no command given)
9paste dashboard
# or just
9paste
```

The dashboard allows you to:
- View and manage recipes
- Add/remove transformations
- Preview transformations in real-time
- Configure settings and hotkeys

### Background Service

```bash
# Start the background service
9paste start
```

The background service:
- Monitors clipboard changes
- Automatically applies the active recipe
- Responds to hotkeys
- Shows system tray icon

### Quick Transformations

Apply a transformation directly from the command line:

```bash
# Apply a recipe
9paste apply "Plain Text"

# Quick transformation
9paste transform lowercase
9paste transform uppercase
9paste transform trim
9paste transform remove-duplicates
9paste transform sort
9paste transform slugify
9paste transform fix-quotes

# View clipboard
9paste show

# List recipes
9paste list

# Toggle auto-transform
9paste toggle
```

## 🎯 Use Cases

### For Academics & Legal Professionals
- Standardize text while preserving essential formatting
- Remove hidden characters from PDFs
- Fix quote characters for citations

### For Data Analysts & Researchers
- Remove duplicate lines
- Sort lists alphabetically
- Strip formatting from spreadsheet pastes

### For Writers & Content Creators
- Enforce consistent capitalization
- Remove emojis from professional documents
- Convert case for headlines

### For Developers & Programmers
- Fix smart quotes that break code
- Normalize whitespace and indentation
- Convert between naming conventions (camelCase ↔ snake_case)

## ⚙️ Configuration

Configuration is stored in:
- **Linux**: `~/.config/9paste/`
- **macOS**: `~/Library/Application Support/9paste/`
- **Windows**: `%APPDATA%\9paste\`

### Files
- `config.json` - Application settings
- `recipes.json` - Your saved recipes
- `history.json` - Clipboard history (if enabled)

## 🛠️ Built-in Recipes

9Paste comes with these default recipes:

| Recipe | Description |
|--------|-------------|
| 📝 Plain Text | Strip formatting, fix quotes, normalize whitespace |
| 💻 Clean Code | Fix smart quotes, trim lines, Unix line endings, tabs to spaces |
| 🔢 Unique Lines | Remove duplicates and empty lines |
| 📊 Sort Lines | Sort alphabetically |
| 🔒 Privacy Mode | Remove emails, phone numbers, URLs |
| 📚 Academic | Clean up text for citations |
| 🚫 No Emoji | Remove all emojis |

## 🔑 Default Hotkeys

| Hotkey | Action |
|--------|--------|
| `Ctrl+Shift+T` | Toggle transformation |
| `Ctrl+Shift+V` | Quick recipe menu |
| `Ctrl+Shift+D` | Open dashboard |

## 🏗️ Architecture

```
9paste/
├── src/
│   ├── main.rs         # CLI entry point
│   ├── lib.rs          # Library exports
│   ├── clipboard.rs    # Clipboard monitoring & transformation
│   ├── recipe.rs       # Recipe definitions & management
│   ├── transformers.rs # Text transformation functions
│   ├── config.rs       # Configuration management
│   ├── dashboard.rs    # GUI dashboard (egui)
│   ├── hotkeys.rs      # Global hotkey handling
│   └── tray.rs         # System tray integration
└── Cargo.toml
```

## 📝 License

MIT License - see [LICENSE](LICENSE) for details.

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

---

**9Paste** - Because your clipboard deserves better. 🔧
