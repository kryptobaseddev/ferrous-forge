# CI/CD Setup Guide

This guide helps you set up the CI/CD pipeline for your fork of Ferrous Forge.

## 🔑 Required Secrets

### 1. Codecov Token (for code coverage)

**Getting the token:**
1. Visit [codecov.io](https://codecov.io)
2. Sign in with GitHub
3. Add your repository (`kryptobaseddev/ferrous-forge`)
4. Copy the `CODECOV_TOKEN` shown on the setup page

**Adding to GitHub:**
1. Go to your repo → Settings → Secrets and variables → Actions
2. Click "New repository secret"
3. Name: `CODECOV_TOKEN`
4. Value: Paste the token from Codecov
5. Click "Add secret"

## 📄 GitHub Pages Setup (for documentation)

Your current settings are correct:
- **Source**: Deploy from a branch
- **Branch**: `gh-pages`
- **Folder**: `/ (root)`

The CI workflow will automatically:
1. Build docs with `cargo doc`
2. Push to `gh-pages` branch
3. GitHub Pages serves from there

## 🔨 Cross-Compilation

The CI now uses a simplified approach that:
- **Checks** that code compiles for Linux, Windows, and musl targets
- **Builds** actual binaries only for Linux
- **Avoids** complex cross-compilation toolchains

This is more reliable than using the `cross` tool which requires Docker and additional setup.

## ✅ CI Pipeline Overview

| Job | Purpose | Required Setup |
|-----|---------|----------------|
| **Check & Lint** | Code formatting and clippy | None |
| **Test** | Run all tests | None |
| **Security Audit** | Check for vulnerabilities | None |
| **Code Coverage** | Measure test coverage | `CODECOV_TOKEN` secret |
| **Benchmarks** | Performance testing | None |
| **Integration** | End-to-end testing | None |
| **Cross Compile** | Multi-platform support | None |
| **Documentation** | Build and deploy docs | GitHub Pages enabled |

## 🚀 Triggering CI

The CI runs automatically on:
- Every push to `main` or `develop` branches
- Every pull request to `main`
- Weekly schedule (Sundays at midnight UTC)

## 📊 Monitoring

- **CI Status**: Check the Actions tab in your GitHub repository
- **Coverage Reports**: View on codecov.io after setup
- **Documentation**: Available at `https://[username].github.io/ferrous-forge/` after first successful run

## 🔍 Troubleshooting

### Codecov Rate Limiting
If you see "Rate limit reached" errors, ensure you've added the `CODECOV_TOKEN` secret.

### Documentation Not Deploying
1. Ensure GitHub Pages is enabled
2. Check that the `gh-pages` branch exists (created after first CI run)
3. Verify the workflow has write permissions

### Cross-Compilation Failures
The simplified setup should work reliably. If issues persist:
- Windows compilation requires MinGW libraries
- musl compilation needs musl-tools
- These are pre-installed on GitHub Actions Ubuntu runners
