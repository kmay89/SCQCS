# Site Security, Privacy & SEO Checklist

A comprehensive checklist for publishing secure, privacy-respecting, SEO-optimized static sites.

> **Free to use.** Copy this checklist to your own projects. No attribution required.
> Works with Netlify, Vercel, Cloudflare Pages, or any static host that supports `_headers`.

Originally developed for [SCQCS](https://scqcs.com), [SecuraCV](https://securacv.netlify.app), and [ERRERLabs](https://github.com/kmay89) projects.

---

## Pre-Deployment Security Audit

### Secrets & Sensitive Data
- [ ] No API keys, tokens, or credentials in code
- [ ] No hardcoded emails (use contact forms or "coming soon" if not ready)
- [ ] No `.env` files or private configuration committed
- [ ] No personal data or PII in repository
- [ ] No commented-out credentials or test data
- [ ] Search entire repo: `grep -r -i "password\|secret\|api.?key\|token\|credential" .`

### Code Hygiene
- [ ] No `console.log()` statements in production
- [ ] No `TODO`, `FIXME`, `XXX`, `HACK` comments
- [ ] No lorem ipsum or placeholder text
- [ ] No developer comments like "NOW 3 ITEMS" or debug notes
- [ ] All external links have `rel="noopener noreferrer"`
- [ ] All anchor links point to valid section IDs

---

## HTTP Security Headers (via `_headers` for Netlify)

### Required Headers
```
X-Frame-Options: DENY
X-Content-Type-Options: nosniff
Referrer-Policy: strict-origin-when-cross-origin
Strict-Transport-Security: max-age=31536000; includeSubDomains; preload
```

### Content Security Policy
```
Content-Security-Policy: default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline' https://fonts.googleapis.com; font-src https://fonts.gstatic.com; img-src 'self' data:; media-src 'self'; frame-ancestors 'none'; base-uri 'self'; form-action 'self'; upgrade-insecure-requests
```

**CSP Best Practices:**
- Define CSP in HTTP headers only (not meta tags) - single source of truth
- Remove `'unsafe-inline'` from `script-src` by using external JS files
- `'unsafe-inline'` for `style-src` is acceptable (lower risk than scripts)
- Use `upgrade-insecure-requests` to auto-upgrade HTTP to HTTPS

### Permissions Policy
```
Permissions-Policy: accelerometer=(), camera=(), geolocation=(), gyroscope=(), magnetometer=(), microphone=(), payment=(), usb=(), interest-cohort=()
```

### Cross-Origin Policies
```
Cross-Origin-Opener-Policy: same-origin
Cross-Origin-Embedder-Policy: credentialless
Cross-Origin-Resource-Policy: same-origin
```

### Deprecated Headers (DO NOT USE)
- ~~X-XSS-Protection~~ - Deprecated, can introduce vulnerabilities. CSP replaces it.

---

## Security Files

### `/.well-known/security.txt` (RFC 9116)
```
# Contact information
Contact: mailto:security@yourdomain.com

Expires: [date one year from now]
Policy: https://yourdomain.com/#legal
Preferred-Languages: en
Canonical: https://yourdomain.com/.well-known/security.txt
```

### `/security.txt` (redirect)
Create a root-level file that references the canonical location, or use `_redirects`:
```
/security.txt /.well-known/security.txt 200
```

### `/humans.txt`
```
/* TEAM */
Organization: YourOrg
Site: https://yourdomain.com

/* SITE */
Language: English
Standards: HTML5, CSS3, ES6+, Schema.org, security.txt
Software: Static site, no server-side processing

/* SECURITY APPROACH */
- Minimal data collection
- No cookies, no analytics, no tracking
- Static content only - minimal attack surface
- Security headers configured
- Content Security Policy enforced
- All resources served over HTTPS
```

---

## SEO Optimization

### Essential Meta Tags
```html
<meta charset="utf-8" />
<meta name="viewport" content="width=device-width, initial-scale=1" />
<meta name="description" content="..." />
<meta name="author" content="YourOrg" />
<meta name="robots" content="index, follow, max-image-preview:large, max-snippet:-1, max-video-preview:-1" />
<link rel="canonical" href="https://yourdomain.com/" />
```

### Open Graph (Facebook, LinkedIn)
```html
<meta property="og:title" content="..." />
<meta property="og:description" content="..." />
<meta property="og:type" content="website" />
<meta property="og:url" content="https://yourdomain.com" />
<meta property="og:image" content="https://yourdomain.com/og-image.jpeg" />
```

### Twitter Card
```html
<meta name="twitter:card" content="summary_large_image" />
<meta name="twitter:title" content="..." />
<meta name="twitter:description" content="..." />
<meta name="twitter:image" content="https://yourdomain.com/og-image.jpeg" />
```

### `/robots.txt`
```
User-agent: *
Allow: /

# AI/LLM Crawlers
User-agent: GPTBot
Allow: /

User-agent: Google-Extended
Allow: /

User-agent: Anthropic-AI
Allow: /

User-agent: PerplexityBot
Allow: /

Sitemap: https://yourdomain.com/sitemap.xml
```

### `/sitemap.xml`
```xml
<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
  <url>
    <loc>https://yourdomain.com/</loc>
    <changefreq>monthly</changefreq>
    <priority>1.0</priority>
  </url>
</urlset>
```

---

## Structured Data (JSON-LD)

### Organization
```json
{
  "@context": "https://schema.org",
  "@type": "Organization",
  "name": "YourOrg",
  "url": "https://yourdomain.com",
  "logo": "https://yourdomain.com/logo.png"
}
```

### WebPage
```json
{
  "@context": "https://schema.org",
  "@type": "WebPage",
  "name": "Page Title",
  "description": "Page description",
  "url": "https://yourdomain.com"
}
```

### SoftwareApplication (for frameworks/tools)
```json
{
  "@context": "https://schema.org",
  "@type": "SoftwareApplication",
  "name": "Your Tool",
  "applicationCategory": "SecurityApplication",
  "description": "...",
  "url": "https://yourdomain.com",
  "isAccessibleForFree": true
}
```

---

## AI/LLM Discoverability

### `/llms.txt`
A plain-text file for AI systems containing:
- Summary of the site/project
- Core concepts explained
- Key terms glossary
- Important disclaimers
- Related projects

See SCQCS llms.txt for a complete example.

---

## Favicons & PWA

### Required Files (in `assets/` directory)
- `assets/favicon.ico` (legacy)
- `assets/favicon-16x16.png`
- `assets/favicon-32x32.png`
- `assets/apple-touch-icon.png` (180x180)
- `assets/android-chrome-192x192.png`
- `assets/android-chrome-512x512.png`
- `site.webmanifest`

### HTML Links
```html
<link rel="apple-touch-icon" sizes="180x180" href="assets/apple-touch-icon.png">
<link rel="icon" type="image/png" sizes="32x32" href="assets/favicon-32x32.png">
<link rel="icon" type="image/png" sizes="16x16" href="assets/favicon-16x16.png">
<link rel="icon" href="assets/favicon.ico">
<link rel="manifest" href="site.webmanifest">
<meta name="theme-color" content="#000000" />
```

---

## Performance

### Resource Hints
```html
<link rel="preconnect" href="https://fonts.googleapis.com" crossorigin>
<link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
<link rel="dns-prefetch" href="https://fonts.googleapis.com">
```

### Caching (via `_headers`)
```
/*.js
  Cache-Control: public, max-age=31536000, immutable

/*.css
  Cache-Control: public, max-age=31536000, immutable

/*.png
  Cache-Control: public, max-age=31536000, immutable
```

---

## Repository Hygiene

### Required Files
- [ ] `README.md` - Project overview
- [ ] `LICENSE` - Clear licensing (MIT recommended for docs)
- [ ] `.gitignore` - OS files, editor files, build artifacts

### `.gitignore` Template
```
.DS_Store
Thumbs.db
.idea/
.vscode/
*.swp
node_modules/
dist/
.env
.env.local
.netlify/
```

---

## Legal Copy Guidelines

### Words to Avoid
| Avoid | Use Instead |
|-------|-------------|
| permanent | durable, tamper-evident |
| immutable | tamper-evident |
| impossible | designed to prevent |
| provable | verifiable |
| guarantee | design intent |
| certified | aligned with, applies patterns from |

### Required Disclaimers
- "This is not legal or security advice"
- "Security depends on correct implementation"
- "No certification claims (FIPS, Common Criteria, etc.)"
- "Provided as-is without warranties"

---

## Compliance Framework Reference

SCQCS patterns support—but do not guarantee—compliance with major regulatory frameworks. Pattern alignment is a starting point, not certification.

### HIPAA
| Requirement | SCQCS Pattern Support |
|-------------|----------------------|
| Audit controls (§164.312(b)) | Append-only logging, tamper-evident records |
| Access controls (§164.312(a)) | Accountable access, break-glass procedures |
| Integrity controls (§164.312(c)) | Sealed storage, cryptographic verification |
| Transmission security (§164.312(e)) | Crypto-agile design, encryption patterns |

**Beyond patterns:** Risk assessment, BAAs, workforce training, physical safeguards, incident response procedures.

### GDPR
| Principle | SCQCS Pattern Support |
|-----------|----------------------|
| Data minimization (Art. 5(1)(c)) | Collect only necessary data by design |
| Purpose limitation (Art. 5(1)(b)) | Architecture enforces declared purposes |
| Accountability (Art. 5(2)) | Audit trails demonstrate compliance |
| Security (Art. 32) | Encryption, access controls, integrity checks |

**Beyond patterns:** Lawful basis documentation, DPO appointment, DPIA process, data subject request workflows, cross-border transfer mechanisms.

### ISO 27001
| Control Objective | SCQCS Pattern Support |
|-------------------|----------------------|
| A.9 Access control | Role-based access, break-glass accountability |
| A.10 Cryptography | Crypto-agile design, key management patterns |
| A.12 Operations security | Comprehensive logging, change management |
| A.18 Compliance | Built-in audit capabilities |

**Beyond patterns:** Complete ISMS, risk assessment methodology, statement of applicability, management review, internal audits, continual improvement process.

### SOC 2 Trust Services Criteria
| Criterion | SCQCS Pattern Support |
|-----------|----------------------|
| Security (CC6) | Access controls, encryption, logging |
| Availability (A1) | Resilience patterns, recovery design |
| Confidentiality (C1) | Sealed storage, minimal data exposure |
| Processing Integrity (PI1) | Tamper-evident logs, verification |
| Privacy (P1-P8) | Data minimization, purpose limitation |

**Beyond patterns:** Control documentation, consistent operation evidence, independent auditor examination, management assertions.

### Common Compliance Checklist
- [ ] Identify applicable regulations for your jurisdiction and data types
- [ ] Map SCQCS patterns to specific regulatory requirements
- [ ] Document gaps between patterns and full compliance
- [ ] Implement organizational controls (policies, training, procedures)
- [ ] Engage qualified legal/compliance professionals
- [ ] Plan for certification audits where required
- [ ] Establish ongoing monitoring and review processes

---

## Pre-Launch Checklist

### Security
- [ ] All secrets removed
- [ ] External links secured with `rel="noopener noreferrer"`
- [ ] CSP configured (header only, not meta tag)
- [ ] HSTS enabled with preload
- [ ] security.txt published
- [ ] No deprecated headers (X-XSS-Protection)

### SEO
- [ ] Canonical URL set to production domain
- [ ] og:url and og:image use production URLs
- [ ] robots.txt references production sitemap
- [ ] sitemap.xml uses production URLs
- [ ] JSON-LD structured data valid

### Privacy
- [ ] No analytics/tracking (or disclosed if present)
- [ ] No cookies (or disclosed if present)
- [ ] Privacy notice in legal section
- [ ] Third-party resources disclosed (e.g., Google Fonts)

### Repository
- [ ] README complete
- [ ] LICENSE file present
- [ ] .gitignore configured
- [ ] No sensitive files committed

---

## Tools for Validation

- **Security Headers**: https://securityheaders.com
- **SSL/TLS**: https://www.ssllabs.com/ssltest/
- **CSP Evaluator**: https://csp-evaluator.withgoogle.com
- **Structured Data**: https://search.google.com/test/rich-results
- **Open Graph**: https://developers.facebook.com/tools/debug/
- **Twitter Card**: https://cards-dev.twitter.com/validator

---

## Authoritative Sources

These recommendations are based on industry standards and official documentation:

### Security Headers
- [MDN: Content-Security-Policy](https://developer.mozilla.org/en-US/docs/Web/HTTP/CSP)
- [MDN: HTTP Security Headers](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers#security)
- [OWASP Secure Headers Project](https://owasp.org/www-project-secure-headers/)
- [Cloudflare: Security Headers](https://developers.cloudflare.com/fundamentals/security/security-headers/)

### Security Best Practices
- [OWASP Top 10](https://owasp.org/www-project-top-ten/)
- [Google Web Fundamentals: Security](https://developers.google.com/web/fundamentals/security)
- [Mozilla Web Security Guidelines](https://infosec.mozilla.org/guidelines/web_security)

### SEO & Structured Data
- [Google Search Central](https://developers.google.com/search/docs)
- [Schema.org](https://schema.org/)
- [Open Graph Protocol](https://ogp.me/)

### Security Disclosure
- [RFC 9116: security.txt](https://www.rfc-editor.org/rfc/rfc9116)
- [securitytxt.org](https://securitytxt.org/)

### Platform Documentation
- [Netlify Headers](https://docs.netlify.com/routing/headers/)
- [Cloudflare Pages Headers](https://developers.cloudflare.com/pages/configuration/headers/)
- [Vercel Headers](https://vercel.com/docs/edge-network/headers)

---

## Adapting This Checklist

This checklist is designed to be copied and modified. To adapt it for your project:

1. **Fork or copy** this file to your repository
2. **Replace domain references** (`scqcs.com` → `yourdomain.com`)
3. **Adjust CSP** for your specific resource needs (fonts, images, APIs)
4. **Add project-specific items** to the pre-launch checklist
5. **Remove sections** that don't apply to your stack

### Hosting Platforms

| Platform | Headers File | Redirects File |
|----------|--------------|----------------|
| Netlify | `_headers` | `_redirects` |
| Vercel | `vercel.json` | `vercel.json` |
| Cloudflare Pages | `_headers` | `_redirects` |
| GitHub Pages | Not supported (use meta tags) | Not supported |

---

## See Also

- [`PREFLIGHT.md`](PREFLIGHT.md) - Quick runnable checks before every deploy
- [`GETTING_STARTED.md`](GETTING_STARTED.md) - Step-by-step Netlify deployment guide
- [`_headers`](../_headers) - Security headers configuration

---

*Maintained by [ERRERLabs](https://github.com/kmay89). Contributions welcome.*

*Free to use, modify, and redistribute. No attribution required.*
