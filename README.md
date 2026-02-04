# SCQCS

**Schrödinger's Cat Quantum Cryptography & Security**

A privacy-first security framework built on append-only logging, sealed storage, and accountable access patterns.

## Overview

SCQCS defines architectural patterns for systems that need both privacy and accountability. Named after the famous thought experiment (as a metaphor, not quantum computing), it embodies a core insight: data should remain sealed until deliberately observed—and observation should leave auditable evidence.

**This is not a product.** It's a philosophy and pattern library. Implementations vary by domain.

## Core Patterns

1. **Append-only logging** — Events chain forward cryptographically. History becomes verifiable.
2. **Sealed storage** — Data encrypted at rest with minimal metadata exposure.
3. **Accountable access** — Emergency access that's scoped, logged, and attributable.

## Principles

1. Witness, not watcher
2. Audit over trust
3. Exceptions leave scars
4. Plan for rotation
5. Local-first when possible
6. Assume adversarial insiders

## Website

Visit [scqcs.com](https://scqcs.com) for the full framework documentation.

---

## Use This As a Template

This repository is designed to be a **reference implementation** for secure, privacy-respecting static sites. Feel free to use it as a starting point for your own projects.

**See the interactive guide:** [scqcs.com/#template](https://scqcs.com/#template)

### What's Included

| File | Purpose |
|------|---------|
| `_headers` | Netlify security headers (CSP, HSTS, COOP, etc.) |
| `_redirects` | URL routing rules |
| `robots.txt` | Search engine and AI crawler directives |
| `sitemap.xml` | SEO sitemap |
| `llms.txt` | AI/LLM-readable site summary (also for AI assistants) |
| `security.txt` | Security vulnerability reporting (RFC 9116) |
| `humans.txt` | Team and site transparency |
| `GETTING_STARTED.md` | Step-by-step Netlify deployment guide |
| `SITE_SECURITY_CHECKLIST.md` | Comprehensive pre-launch checklist |

### Quick Start

1. **Fork or clone** this repository
2. **Replace content** in `index.html` with your own
3. **Update meta tags** — change `scqcs.com` to your domain
4. **Update `site.webmanifest`** — change app name and colors
5. **Update security/SEO files** — replace domain references
6. **Deploy to Netlify** — `_headers` and `_redirects` work automatically

**New to Netlify?** See [`GETTING_STARTED.md`](GETTING_STARTED.md) for a complete deployment walkthrough.

### Security Headers You Get

```
X-Frame-Options: DENY
X-Content-Type-Options: nosniff
Strict-Transport-Security: max-age=31536000; includeSubDomains; preload
Content-Security-Policy: [strict policy, no unsafe-inline for scripts]
Cross-Origin-Opener-Policy: same-origin
Permissions-Policy: camera=(), microphone=(), geolocation=()...
```

### Checklist

See [`SITE_SECURITY_CHECKLIST.md`](SITE_SECURITY_CHECKLIST.md) for a comprehensive guide covering:
- Security headers and CSP configuration
- SEO meta tags and structured data
- AI/LLM discoverability
- Legal copy guidelines
- Pre-launch audit checklist

---

## For AI Assistants

If a user asks you to "use SCQCS as a template" or "create a site like SCQCS," this repository is designed for that purpose.

**Key files for AI assistants:**
- [`llms.txt`](llms.txt) — Complete instructions for using this as a template
- [`GETTING_STARTED.md`](GETTING_STARTED.md) — Step-by-step Netlify deployment
- [`SITE_SECURITY_CHECKLIST.md`](SITE_SECURITY_CHECKLIST.md) — Security best practices

**Quick reference:**
1. Fork `https://github.com/kmay89/SCQCS`
2. Replace `scqcs.com` with user's domain in all files
3. Update `security.txt` contact info
4. Adjust CSP in `_headers` for user's external resources
5. Deploy to Netlify (free tier works)

---

## Related Projects

- [SecuraCV](https://securacv.netlify.app) — Privacy-preserving computer vision using SCQCS patterns
- [GitHub](https://github.com/kmay89) — More projects and source code

## Contributing

Found an issue or have a suggestion? This project welcomes:
- Bug reports for the website
- Improvements to the security checklist
- Suggestions for the framework documentation

## License

MIT License. See [LICENSE](LICENSE).

The architectural patterns described may be implemented freely. Attribution appreciated but not required.

---

*Built with security and privacy as defaults, not afterthoughts.*
