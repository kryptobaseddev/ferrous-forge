# Integration Guide

## IDE Integration

Ferrous Forge can be integrated with popular IDEs and editors to provide real-time feedback and automatic standards enforcement.

### Visual Studio Code

#### Method 1: Ferrous Forge Extension (Coming Soon)
We're developing an official VS Code extension. In the meantime, use Method 2.

#### Method 2: Manual Configuration
Add to your `.vscode/settings.json`:

```json
{
  "rust-analyzer.checkOnSave.command": "ferrous-forge",
  "rust-analyzer.checkOnSave.extraArgs": ["validate", "--quiet"],
  "rust-analyzer.procMacro.enable": true,
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
      "command": "ferrous-forge validate",
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

#### File Watcher

1. Install File Watchers plugin
2. Go to **Settings** → **Tools** → **File Watchers**
3. Add new watcher:
   - **File type**: Rust
   - **Program**: `ferrous-forge`
   - **Arguments**: `validate --quiet`
   - **Output filters**: Parse Ferrous Forge output

### Vim / Neovim

#### Using ALE (Asynchronous Lint Engine)

Add to your `.vimrc` or `init.vim`:

```vim
" Add Ferrous Forge as a linter
let g:ale_linters = {
\   'rust': ['cargo', 'clippy', 'ferrous-forge'],
\}

" Custom Ferrous Forge linter definition
call ale#linter#Define('rust', {
\   'name': 'ferrous-forge',
\   'executable': 'ferrous-forge',
\   'command': 'ferrous-forge validate --quiet %s',
\   'callback': 'ale#handlers#rust#HandleRustErrors',
\})

" Run on save
let g:ale_lint_on_save = 1
let g:ale_fix_on_save = 1
```

#### Using nvim-lspconfig (Neovim only)

```lua
-- Add to your Neovim configuration
local nvim_lsp = require('lspconfig')

-- Custom Ferrous Forge integration
nvim_lsp.rust_analyzer.setup{
  on_attach = function(client, bufnr)
    -- Run Ferrous Forge on save
    vim.api.nvim_create_autocmd("BufWritePre", {
      buffer = bufnr,
      callback = function()
        vim.fn.system('ferrous-forge validate --quiet')
      end
    })
  end,
  settings = {
    ["rust-analyzer"] = {
      checkOnSave = {
        command = "clippy",
      },
    },
  },
}
```

### Emacs

#### Using Flycheck

Add to your Emacs configuration:

```elisp
;; Define Ferrous Forge checker
(flycheck-define-checker ferrous-forge
  "Ferrous Forge standards checker for Rust"
  :command ("ferrous-forge" "validate" "--json" source)
  :error-parser flycheck-parse-json
  :modes rust-mode
  :next-checkers ((warning . rust-clippy)))

;; Add to Rust mode hook
(add-hook 'rust-mode-hook
          (lambda ()
            (flycheck-add-next-checker 'rust-cargo 'ferrous-forge)))
```

#### Using LSP Mode

```elisp
(use-package lsp-mode
  :hook (rust-mode . lsp)
  :config
  (setq lsp-rust-analyzer-cargo-watch-command "ferrous-forge validate"))
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
      
      - name: Check Documentation Coverage
        run: ferrous-forge validate --check-docs
      
      - name: Security Audit
        run: ferrous-forge validate --security
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
    - ferrous-forge validate --check-docs
    - ferrous-forge validate --security
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
    
    stage('Documentation') {
      steps {
        sh 'ferrous-forge validate --check-docs'
      }
    }
    
    stage('Security') {
      steps {
        sh 'ferrous-forge validate --security'
      }
    }
  }
  
  post {
    always {
      publishHTML([
        reportDir: 'target/ferrous-forge',
        reportFiles: 'report.html',
        reportName: 'Ferrous Forge Report'
      ])
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
      - run:
          name: Check Documentation
          command: ferrous-forge validate --check-docs
      - run:
          name: Security Audit
          command: ferrous-forge validate --security
      - store_artifacts:
          path: target/ferrous-forge/report.html
          destination: ferrous-forge-report

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

# Override standard commands
build = "!ferrous-forge validate --quiet && cargo build"
test = "!ferrous-forge validate --quiet && cargo test"
```

### Make Integration

Create `Makefile`:

```makefile
.PHONY: validate build test clean

validate:
	@ferrous-forge validate

build: validate
	@cargo build --release

test: validate
	@cargo test

clean:
	@cargo clean
	@ferrous-forge clean

install:
	@cargo install --path .

check: validate test
	@echo "✅ All checks passed!"
```

## Docker Integration

### Dockerfile with Ferrous Forge

```dockerfile
FROM rust:1.85 as builder

# Install Ferrous Forge
RUN cargo install ferrous-forge

WORKDIR /app
COPY . .

# Validate before building
RUN ferrous-forge validate
RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/myapp /usr/local/bin/
CMD ["myapp"]
```

### Docker Compose

```yaml
version: '3.8'

services:
  validator:
    image: rust:1.85
    volumes:
      - .:/app
    working_dir: /app
    command: |
      sh -c "
        cargo install ferrous-forge &&
        ferrous-forge validate --watch
      "
```

## Pre-commit Hook Integration

### Using pre-commit framework

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

Install the hook:
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
alias ffc='ferrous-forge validate --check-docs'
alias ffs='ferrous-forge validate --security'
alias ffu='ferrous-forge update'

# Function for quick validation
ff() {
  ferrous-forge validate "$@" && echo "✅ Validation passed!"
}
```

### Fish Shell

Add to `~/.config/fish/config.fish`:

```fish
# Ferrous Forge abbreviations
abbr ffv 'ferrous-forge validate'
abbr ffc 'ferrous-forge validate --check-docs'
abbr ffs 'ferrous-forge validate --security'

function ff
  ferrous-forge validate $argv; and echo "✅ Validation passed!"
end
```

## API Integration

### REST API Wrapper (Coming Soon)

```bash
# Start Ferrous Forge API server
ferrous-forge serve --port 8080

# Validate via API
curl -X POST http://localhost:8080/validate \
  -H "Content-Type: application/json" \
  -d '{"path": "/path/to/project"}'
```

## Troubleshooting Integration Issues

### Common Problems

1. **IDE not recognizing ferrous-forge command**
   - Ensure `~/.cargo/bin` is in your PATH
   - Restart your IDE after installation

2. **CI/CD timeouts**
   - Use `--quiet` flag for faster execution
   - Cache Ferrous Forge installation

3. **Conflict with existing tools**
   - Adjust tool priority in IDE settings
   - Use `--no-deps` flag to skip dependency checks

For more help, see [Troubleshooting Guide](troubleshooting.md).