# Integration Guide

Integrating Ferrous Forge with IDEs, CI/CD, and development workflows.

## IDE Integration

### Visual Studio Code

#### Manual Configuration

Add to `.vscode/settings.json`:

```json
{
  "rust-analyzer.checkOnSave.command": "clippy",
  "rust-analyzer.checkOnSave.extraArgs": ["--all-targets", "--all-features"],
  "editor.formatOnSave": true,
  "[rust]": {
    "editor.defaultFormatter": "rust-lang.rust-analyzer"
  }
}
```

#### Custom Tasks

Create `.vscode/tasks.json`:

```json
{
  "version": "2.0.0",
  "tasks": [
    {
      "label": "Ferrous Forge Validate",
      "type": "shell",
      "command": "ferrous-forge",
      "args": ["validate"],
      "group": {
        "kind": "test",
        "isDefault": true
      },
      "presentation": {
        "reveal": "always",
        "panel": "new"
      },
      "problemMatcher": "$rustc"
    }
  ]
}
```

**Coming Soon:** VS Code extension with real-time validation

### IntelliJ IDEA / CLion

#### External Tool Configuration

1. Go to **Settings** → **Tools** → **External Tools**
2. Click **+** to add a new tool
3. Configure:
   - **Name**: Ferrous Forge Validate
   - **Program**: `ferrous-forge`
   - **Arguments**: `validate $ProjectFileDir$`
   - **Working directory**: `$ProjectFileDir$`
4. Add keyboard shortcut in **Settings** → **Keymap**

### Vim / Neovim

#### Using ALE (Asynchronous Lint Engine)

Add to `.vimrc` or `init.vim`:

```vim
" Add Ferrous Forge as a linter
let g:ale_linters = {
\   'rust': ['cargo', 'clippy'],
\}

" Run Ferrous Forge manually
nnoremap <leader>ff :!ferrous-forge validate --quiet<CR>
```

#### Using nvim-lspconfig

```lua
-- Run Ferrous Forge on save
vim.api.nvim_create_autocmd("BufWritePost", {
  pattern = "*.rs",
  callback = function()
    vim.fn.system('ferrous-forge validate --quiet')
  end
})
```

## CI/CD Integration

### GitHub Actions

Create `.github/workflows/ferrous-forge.yml`:

```yaml
name: Ferrous Forge Validation

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

jobs:
  validate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy
      
      - name: Install Ferrous Forge
        run: cargo install ferrous-forge
      
      - name: Run Ferrous Forge Validation
        run: ferrous-forge validate
```

### GitLab CI

Add to `.gitlab-ci.yml`:

```yaml
ferrous-forge:
  stage: test
  image: rust:latest
  before_script:
    - cargo install ferrous-forge
  script:
    - ferrous-forge validate
  cache:
    paths:
      - target/
      - .cargo/
```

### Jenkins

Create `Jenkinsfile`:

```groovy
pipeline {
  agent any
  
  stages {
    stage('Setup') {
      steps {
        sh 'cargo install ferrous-forge'
      }
    }
    
    stage('Validate') {
      steps {
        sh 'ferrous-forge validate'
      }
    }
  }
}
```

### CircleCI

Add to `.circleci/config.yml`:

```yaml
version: 2.1

jobs:
  ferrous-forge:
    docker:
      - image: rust:latest
    steps:
      - checkout
      - run:
          name: Install Ferrous Forge
          command: cargo install ferrous-forge
      - run:
          name: Validate Code
          command: ferrous-forge validate

workflows:
  version: 2
  validate:
    jobs:
      - ferrous-forge
```

## Build System Integration

### Cargo Configuration

Add to `.cargo/config.toml`:

```toml
[alias]
# Add Ferrous Forge aliases
ff-validate = "run --bin ferrous-forge -- validate"
ff-check = "run --bin ferrous-forge -- validate --quiet"
ff-fix = "run --bin ferrous-forge -- fix"
```

### Make Integration

Create `Makefile`:

```makefile
.PHONY: validate build test

validate:
	@ferrous-forge validate

build: validate
	@cargo build --release

test: validate
	@cargo test

check: validate test
	@echo "✅ All checks passed!"
```

## Pre-commit Hooks

Ferrous Forge installs git hooks automatically when you run:

```bash
ferrous-forge init --project
# or
ferrous-forge safety install
```

The hooks validate:
- Code formatting (`cargo fmt`)
- Linting (`cargo clippy`)
- Ferrous Forge standards

### Using pre-commit Framework

Create `.pre-commit-config.yaml`:

```yaml
repos:
  - repo: local
    hooks:
      - id: ferrous-forge
        name: Ferrous Forge Validation
        entry: ferrous-forge validate
        language: system
        files: '\.rs$'
        pass_filenames: false
```

Install:
```bash
pip install pre-commit
pre-commit install
```

## Shell Integration

### Bash/Zsh Aliases

Add to `~/.bashrc` or `~/.zshrc`:

```bash
# Ferrous Forge aliases
alias ffv='ferrous-forge validate'
alias ffc='ferrous-forge fix'
alias ffs='ferrous-forge safety check'
alias fft='ferrous-forge template list'

# Quick validation function
ff() {
  ferrous-forge validate "$@" && echo "✅ Validation passed!"
}
```

### Fish Shell

Add to `~/.config/fish/config.fish`:

```fish
# Ferrous Forge abbreviations
abbr ffv 'ferrous-forge validate'
abbr ffc 'ferrous-forge fix'
abbr ffs 'ferrous-forge safety check'

function ff
  ferrous-forge validate $argv; and echo "✅ Validation passed!"
end
```

## Troubleshooting Integration

### IDE Not Finding ferrous-forge

Ensure `~/.cargo/bin` is in your PATH:
```bash
# Check PATH
echo $PATH | grep ".cargo/bin"

# Add if missing
echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

### CI/CD Timeouts

Use `--quiet` for faster execution:
```yaml
- run: ferrous-forge validate --quiet
```

### Hook Bypass

Skip hooks in emergencies:
```bash
git commit --no-verify
```

---

**Planned for v2.0:**
- VS Code extension with real-time validation
- IntelliJ plugin
- Language Server Protocol (LSP) support
- REST API for validation
