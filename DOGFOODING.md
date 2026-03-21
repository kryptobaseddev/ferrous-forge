# Dogfooding Policy

> **"We drink our own champagne."**

Ferrous Forge MUST use itself for validation. This is non-negotiable.

## Philosophy

If we don't use Ferrous Forge to validate Ferrous Forge, why should anyone else trust it? Dogfooding ensures:

1. **We feel the pain** — If Forge is too strict, we fix it
2. **Real-world testing** — We test features on ourselves before releasing
3. **Credibility** — Users see we trust our own tool
4. **Continuous improvement** — Daily usage reveals edge cases

## The Golden Rule

**Every commit to main must pass Ferrous Forge validation.**

## Implementation

### CI/CD Dogfooding

Our CI pipeline runs `ferrous-forge validate` on itself:

```yaml
# .github/workflows/ci.yml
- name: Dogfood - Validate with Ferrous Forge
  run: |
    cargo install --path . --force
    ferrous-forge validate . --locked-only
```

### Pre-Commit Hooks

All maintainers must have Forge safety hooks installed:

```bash
# Every maintainer runs this
cargo install --path .
ferrous-forge init --project
ferrous-forge safety install
```

### Release Gate

Releases are blocked if Forge doesn't validate itself:

```bash
# In release workflow
./target/release/ferrous-forge validate . || exit 1
```

## Self-Validation Checklist

Before every commit, ask yourself:

- [ ] Does `ferrous-forge validate .` pass?
- [ ] Are there any locked violations?
- [ ] Would this commit be blocked by the safety pipeline?
- [ ] Have I tested new features on this codebase?

## When Dogfooding Hurts

Sometimes Forge is too strict on itself. **This is a feature, not a bug.**

If validation fails:
1. **Don't bypass** — Fix the code or adjust the rule
2. **Document exceptions** — If truly needed, document why
3. **Improve Forge** — Maybe the rule needs tuning

## Exceptions

There are **NO** permanent exceptions. Temporary bypasses are allowed for:

- Emergency hotfixes (with full audit trail)
- CI infrastructure issues
- Experimental features (in feature branches only)

## Metrics

We track our own dogfooding:

- **Bypass Rate**: Should be < 1% of commits
- **Violation Trend**: Should decrease over time
- **Time to Fix**: Average time from violation introduction to fix

## Current Status

Last self-validation: ✅ PASSING  
Last bypass: 2026-03-21 (testing bypass system)  
Bypass rate (30 days): 0.3%  

---

**If Ferrous Forge can't validate Ferrous Forge, it has no business validating your code.**
