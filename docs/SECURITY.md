# Security Policy

## Reporting Security Vulnerabilities

We take the security of rfmt seriously. If you discover a security vulnerability, please report it to us privately.

### How to Report

1. **Email**: Send details to fujitanisora0414@gmail.com
2. **Subject**: Include "SECURITY" in the subject line
3. **Details**: Provide:
   - Description of the vulnerability
   - Steps to reproduce
   - Potential impact
   - Suggested fix (if any)

### Response Timeline

- **Initial Response**: Within 48 hours
- **Status Update**: Within 7 days
- **Fix Release**: Varies by severity (critical issues prioritized)

## Security Measures

### Input Validation

rfmt implements comprehensive input validation to protect against malicious inputs:

#### File Path Validation
- ✅ Path existence verification
- ✅ File type verification (must be a file, not directory)
- ✅ Canonical path resolution (prevents path traversal)
- ✅ Symbolic link detection (disabled by default)
- ✅ File extension validation (.rb, .rake, .ru, Gemfile, etc.)

#### Source Code Validation
- ✅ Size limits (default: 10MB)
- ✅ UTF-8 encoding validation
- ✅ Null byte detection

### Resource Limits

rfmt enforces resource limits to prevent denial-of-service attacks:

#### File Processing Limits
- **Maximum File Size**: 10MB (configurable)
- **Maximum Memory Usage**: 100MB (configurable)
- **Processing Timeout**: 30 seconds (configurable)
- **Maximum Recursion Depth**: 1000 (configurable)
- **Maximum Threads**: 4 (configurable)

#### Security Policies

rfmt provides three security policy presets:

```rust
// Default policy (balanced)
let policy = SecurityPolicy::default();

// Strict policy (tighter limits)
let policy = SecurityPolicy::strict();

// Permissive policy (relaxed limits)
let policy = SecurityPolicy::permissive();
```

### Error Handling

#### Information Disclosure Prevention
- ✅ Sensitive information sanitized from error messages
- ✅ Absolute paths replaced with relative paths
- ✅ Home directories replaced with `~`
- ✅ Internal errors provide actionable guidance without exposing system details

#### Panic Handling
- ✅ Graceful error recovery
- ✅ No panics propagated to Ruby runtime
- ✅ Comprehensive error types with context

### Dependency Security

#### Dependency Management
- ✅ Regular dependency updates
- ✅ Minimal dependency footprint
- ✅ Trusted dependencies only (official Rust crates)
- ✅ Security audit tools integration

#### Monitoring
- Automated dependency vulnerability scanning (GitHub Dependabot)
- Regular `cargo audit` runs in CI/CD
- `bundle-audit` for Ruby dependencies

## Security Best Practices

### For Users

1. **Keep rfmt Updated**
   ```bash
   gem update rfmt
   ```

2. **Use Default Security Settings**
   - Don't disable security features unless necessary
   - Avoid using permissive security policies in untrusted environments

3. **Validate Input Sources**
   - Only format Ruby files from trusted sources
   - Be cautious with user-submitted code

4. **Monitor Resource Usage**
   - Review logs for unusual activity
   - Set appropriate resource limits for your environment

### For Contributors

1. **Security Review**
   - All PRs undergo security review
   - Use `cargo clippy` for static analysis
   - Run `cargo audit` before submitting

2. **Testing**
   - Write tests for security-critical code
   - Include edge cases and boundary conditions
   - Test with malicious inputs

3. **Documentation**
   - Document security implications of changes
   - Update this file when security features change

## Known Limitations

### Current Scope

rfmt is a code formatter and does NOT:
- ❌ Execute Ruby code
- ❌ Eval user input
- ❌ Make network requests
- ❌ Access files outside the specified paths
- ❌ Modify system configuration

### Platform-Specific Features

- **Memory Tracking**: Only available on Linux (via `/proc/self/status`)
- **Symbolic Link Detection**: Platform-dependent behavior

## Security Audit History

| Date | Version | Auditor | Findings | Status |
|------|---------|---------|----------|--------|
| 2025-01 | 0.1.0 | Internal | Initial implementation | ✅ Resolved |

## Compliance

### Standards
- Follows Rust security best practices
- Implements OWASP secure coding guidelines
- Uses safe Rust (minimal `unsafe` code)

### Unsafe Code
rfmt minimizes the use of `unsafe` Rust:
- ✅ FFI boundary (Magnus) - Required for Ruby integration
- ✅ All unsafe code is documented and reviewed

## Security Checklist

Before each release, we verify:

- [ ] All dependencies updated to latest secure versions
- [ ] `cargo audit` passes with zero vulnerabilities
- [ ] `bundle-audit` passes with zero vulnerabilities
- [ ] All security tests pass
- [ ] No new `unsafe` code without review
- [ ] Documentation reflects current security features
- [ ] Error messages don't leak sensitive information

## Contact

- **Security Issues**: fujitanisora0414@gmail.com
- **General Issues**: https://github.com/fs0414/rfmt/issues
- **Discussions**: https://github.com/fs0414/rfmt/discussions

## Acknowledgments

We thank the security researchers and contributors who help keep rfmt secure.

---

**Last Updated**: 2025-01
**Security Policy Version**: 1.0
