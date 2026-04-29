# Security Disclaimer

## Important notice

**dist_agent_lang v1.0.xx** is in **beta**. This document describes the current security posture and how to use the software safely. The software is not production-ready for high-value or safety-critical use.

---

## Current security status

### Implemented security features

1. **Reentrancy protection**
   - `ReentrancyGuard` in the runtime prevents re-entry into the same method/contract combination.
   - The `@secure` attribute enforces reentrancy protection plus authentication and audit logging for decorated services/methods.

2. **Safe math**
   - `SafeMath` in the runtime (overflow/underflow, division by zero) for arithmetic in DAL execution.
   - Used by the interpreter for arithmetic operations.

3. **State isolation**
   - State isolation support in the runtime for contract/service state separation.

4. **Cross-chain security**
   - Cross-chain security utilities and signature/validator handling where applicable.

5. **Advanced security / MEV awareness**
   - `@advanced_security` attribute for MEV-related detection and blocking of unprotected execution patterns where configured.

6. **Authentication and authorization**
   - Auth and capability-based access control in the stdlib.
   - Session and caller context; `@secure` enforces authenticated caller.

7. **Agent and server security**
   - **Shell trust**: Agent shell execution (`sh::run`) is gated by `[agent.sh]` config and `DAL_AGENT_SHELL_TRUST` (off / sandboxed / confirmed / trusted).
   - **Capabilities**: Agent capabilities and types constrain what agents can do.
   - **Working root**: Request path length capped to mitigate allocation abuse (e.g. `working_root` in agent serve).
   - **Persistence**: Agent runtime persistence (file/SQLite) is local; no built-in encryption at rest — protect the host and storage.

### Testing and audits

- **Tests**: 1000+ tests across lexer, parser, runtime, stdlib, agent, fleet, mold, security, integration, and HTTP server.
- **Dependency audit**: Run `cargo audit` (or your usual process) for third-party crate vulnerabilities.
- **No formal verification**: Critical smart contracts should undergo formal verification before high-value use.
- **No third-party security audit**: The codebase has not yet had an independent security audit.

### Execution model

DAL runs on a **tree-walking interpreter** (Rust host). There is no JIT or AOT compilation of DAL code. Transpilation backends (blockchain, WASM, native) produce scaffolding or separate artifacts; the primary execution path is interpreted. Security guarantees are those of the runtime and stdlib as implemented, plus the Rust host.

---

## Known limitations

1. **Beta status**
   - Limited real-world deployment and edge-case coverage.
   - Some areas may have thin wiring, TODOs, or incomplete behavior; review the code for your use case.

2. **Agent server**
   - One agent per process for the HTTP serve path. Shell execution is configurable but must be locked down (e.g. sandboxed/confirmed) for untrusted input.
   - Protect API keys and credentials (e.g. LLM, chain) via environment or secure config; do not commit them.

3. **Smart contracts and chain**
   - Transpilation to Solidity and on-chain deployment are supported; the resulting contracts should be audited and tested on testnets before mainnet or high-value use.
   - Reentrancy and safe-math protections apply in the DAL runtime; on-chain behavior depends on the compiled contract and chain.

4. **Formal verification and penetration testing**
   - Not performed. Recommended for critical or high-value applications.

---

## Usage recommendations

### Generally acceptable for

- Development and prototyping
- Agentic and trust-level experimentation
- Locally hosted surfaces and applications
- Testing and validation (including agent and chain workflows)

### Use with caution

- **Production financial applications** — Prefer after 1.1.0+ and additional validation.
- **High-value smart contracts** — Third-party audit and testnet validation recommended.
- **Critical infrastructure** — Additional hardening and comprehensive audit/review recommended before production.
- **Sensitive data** — Ensure compliance, encryption, and access control (e.g. credentials, PII) at the application and host level.

---

## Security best practices

- Run the test suite: `cargo test`, and where applicable `cargo test --all-features`.
- Use `@secure` (and, where relevant, `@advanced_security`) for services that need reentrancy protection and access control.
- Configure agent shell trust explicitly (`[agent.sh]` / `DAL_AGENT_SHELL_TRUST`); avoid `trusted` for untrusted or open inputs.
- Validate and bound inputs (e.g. path lengths, sizes) where they affect allocation or behavior.
- Keep dependencies and the Rust toolchain updated; run `cargo audit` and address known vulnerabilities.
- For production or high-value use: third-party audit, formal verification of critical contracts, and penetration testing as appropriate.

---

## Reporting security issues

If you find a security vulnerability, please report it responsibly:

- **Email**: jason.dinh.developer@gmail.com
- **GitHub**: [Security / issues](https://github.com/okjason-source/dist_agent_lang/issues) (prefer private disclosure until a fix or advisory is ready)
- **Do not** disclose the issue publicly before coordination.

---

## License

This software is provided “as is” without warranty of any kind. See [LICENSE](LICENSE) for full terms. The project is under the Apache License 2.0 and may also be offered under a commercial license; see LICENSE for the dual-licensing notice.

---

**Last updated**: 2026-03  
**Version**: 1.0.xx  
**Status**: Beta
