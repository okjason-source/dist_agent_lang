# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 1.0.x   | :white_check_mark: |
| < 1.0   | :x:                |

## Reporting a Vulnerability

**Please do not report security vulnerabilities through public GitHub issues.**

Instead, please report them via one of the following methods:

1. **Email:** jason.dinh.developer@gmail.com
2. **GitHub Security Advisory:** Use the "Report a vulnerability" button on the Security tab
3. **Private Discussion:** Create a private GitHub Discussion

## What to Include

When reporting a vulnerability, please include:

- Description of the vulnerability
- Steps to reproduce
- Potential impact
- Suggested fix (if any)
- Affected versions

## Response Time

We aim to:
- Acknowledge receipt within **48 hours**
- Provide initial assessment within **7 days**
- Provide regular updates on progress
- Notify you when the vulnerability is fixed

## Disclosure Policy

- We will acknowledge receipt of your vulnerability report
- We will work with you to understand and resolve the issue
- We will notify you when the vulnerability is fixed
- We will credit you in the security advisory (if desired)
- We will coordinate public disclosure timing with you

## Security Best Practices

For users of dist_agent_lang:

- **Always use the latest version** - Security fixes are included in updates
- **Review security advisories** - Check the Security tab regularly
- **Report suspicious behavior** - If something seems wrong, report it
- **Follow security best practices** - Validate input, use safe defaults, avoid unsafe code
- **Keep dependencies updated** - Review and update dependencies regularly

## Security Features

dist_agent_lang includes:

- ✅ Reentrancy protection
- ✅ Safe math (overflow/underflow protection)
- ✅ State isolation
- ✅ Cross-chain security
- ✅ Oracle security (signed feeds, multi-source validation)
- ✅ Transaction atomicity (ACID guarantees)
- ✅ Enhanced security logging with source tracking
- ✅ Comprehensive test coverage (140+ tests)

## Known Limitations

**Beta Release Notice:** dist_agent_lang v1.0.x is currently in beta. While it includes comprehensive security features, it has not yet undergone extensive real-world production testing or third-party security audits.

**Recommended for:**
- Development & Prototyping
- Learning & Experimentation
- Non-Critical Applications
- Testing & Validation

**Use with caution for:**
- Production Financial Applications (wait for v1.1.0+)
- High-Value Smart Contracts (third-party audit recommended)
- Critical Infrastructure (additional validation needed)

## Security Updates

Security updates are released as:
- **Patch releases** (1.0.x) - Critical security fixes
- **Minor releases** (1.x.0) - Security improvements and features
- **Major releases** (x.0.0) - Significant changes, may include security updates

Check the [CHANGELOG.md](CHANGELOG.md) for security-related updates.

## Credits

We thank security researchers who responsibly disclose vulnerabilities. Contributors will be credited in security advisories (with permission).

---

**Thank you for helping keep dist_agent_lang secure!**
