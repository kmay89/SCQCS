# Site Security, Privacy & SEO Checklist

A comprehensive checklist for publishing secure, privacy-respecting, SEO-optimized static sites. Applicable to ERRERLabs, SCQCS, SecuraCV, and similar projects.

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

### Required Files
- `favicon.ico` (legacy)
- `favicon-16x16.png`
- `favicon-32x32.png`
- `apple-touch-icon.png` (180x180)
- `android-chrome-192x192.png`
- `android-chrome-512x512.png`
- `site.webmanifest`

### HTML Links
```html
<link rel="apple-touch-icon" sizes="180x180" href="apple-touch-icon.png">
<link rel="icon" type="image/png" sizes="32x32" href="favicon-32x32.png">
<link rel="icon" type="image/png" sizes="16x16" href="favicon-16x16.png">
<link rel="icon" href="favicon.ico">
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

*Last updated: 2026-02-04*
*Applies to: SCQCS, SecuraCV, ERRERLabs, and related projects*
