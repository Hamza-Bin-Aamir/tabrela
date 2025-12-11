# Pre-commit Hooks Setup

This project uses [pre-commit](https://pre-commit.com/) to ensure code quality before commits.

## Installation

### 1. Install pre-commit framework

**Using pip (recommended):**
```bash
pip install pre-commit
```

**Using system package manager:**
```bash
# Fedora/RHEL
sudo dnf install pre-commit

# Ubuntu/Debian
sudo apt install pre-commit

# macOS
brew install pre-commit
```

### 2. Install the git hook scripts

```bash
cd /path/to/tabrela
pre-commit install
```

### 3. (Optional) Run against all files

```bash
pre-commit run --all-files
```

## What Gets Checked

### Rust Services (auth, attendance, merit, tabulation)
- ✅ **cargo fmt** - Code formatting
- ✅ **cargo check** - Compilation errors
- ✅ **cargo clippy** - Linting and best practices
- ✅ **cargo audit** - Security vulnerabilities (warning only)

### Frontend (web/)
- ✅ **TypeScript** - Type checking
- ✅ **ESLint** - Code linting
- ✅ **Build** - Production build validation

### General
- ✅ Trailing whitespace removal
- ✅ End-of-file fixer
- ✅ YAML/TOML/JSON validation
- ✅ Merge conflict detection
- ✅ Large file prevention (>1MB)

## Usage

### Normal commits
Just commit as usual. The hooks will run automatically:
```bash
git commit -m "Your message"
```

### Bypass hooks (not recommended)
If you need to skip checks:
```bash
git commit --no-verify -m "Your message"
```

### Run hooks manually
```bash
# Run all hooks on staged files
pre-commit run

# Run all hooks on all files
pre-commit run --all-files

# Run specific hook
pre-commit run cargo-fmt --all-files
```

### Update hooks
```bash
pre-commit autoupdate
```

## Troubleshooting

### "command not found: cargo-audit"
Install cargo-audit:
```bash
cargo install cargo-audit
```

### Hooks are slow
Pre-commit caches results. First run is slow, subsequent runs are fast.

### Skip specific hooks
Temporarily skip a hook:
```bash
SKIP=cargo-clippy git commit -m "WIP"
```

### Clean pre-commit cache
```bash
pre-commit clean
```

## Alternative: Bash Script (Legacy)

If you prefer the bash script approach, use:
```bash
./scripts/install-hooks.sh
```

This installs `scripts/pre-commit.sh` as a git hook. However, the pre-commit framework is recommended for better:
- Performance (caching)
- Configurability
- Community support
- Cross-platform compatibility
