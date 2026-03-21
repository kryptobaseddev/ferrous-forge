# Security Policy

## Supported Versions

We release patches for security vulnerabilities. Which versions are eligible
receiving such patches depend on the CVSS v3.0 Rating:

| Version | Supported          |
| ------- | ------------------ |
| 1.7.x   | :white_check_mark: |
| 1.6.x   | :white_check_mark: |
| 1.5.x   | :x:                |
| < 1.5   | :x:                |

## Reporting a Vulnerability

Please report security vulnerabilities to **kryptobaseddev@gmail.com**.

Please include:
- A description of the vulnerability
- Steps to reproduce the issue
- Possible impact of the vulnerability
- Suggested fix (if any)

We will acknowledge receipt within 48 hours and provide a more detailed response
within 7 days indicating the next steps in handling your report.

After the initial reply to your report, the security team will endeavor to keep
you informed of the progress towards a fix and full announcement, and may ask
for additional information or guidance.

## Security Best Practices

When using Ferrous Forge:

1. **Keep your Rust toolchain updated** — Use `ferrous-forge rust update` regularly
2. **Run security audits** — Enable the safety pipeline to run `cargo audit`
3. **Review bypasses** — Regularly check `ferrous-forge safety audit` for unexpected bypasses
4. **Validate templates** — When fetching templates from external sources, validate them first

## Security Features

Ferrous Forge includes several security-focused features:

- **Dependency Scanning**: Integration with `cargo audit` for vulnerability detection
- **Unsafe Code Prevention**: `unsafe_code = "forbid"` enforced by default
- **Lock File Validation**: Ensures dependencies haven't been tampered with
- **Audit Logging**: All bypasses and critical configuration changes are logged

## Disclosure Policy

When we receive a security bug report, we will:

1. Confirm the problem and determine the affected versions
2. Audit code to find any similar problems
3. Prepare fixes for all supported versions
4. Release new versions as quickly as possible
5. Publicly disclose the issue after all supported versions have been patched

## Comments on this Policy

If you have suggestions on how this process could be improved, please submit a
pull request or open an issue to discuss.
