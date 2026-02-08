# Preflight Checklist

A quick, runnable audit to perform before publishing your site.

> **Run these checks** right before going live. They catch the things you forget at 2am.

---

## Quick Command Checks

Run these from your project root:

### 1. No Debug Code Left Behind
```bash
grep -r -i "TODO\|FIXME\|XXX\|console\.log\|debugger" --include="*.html" --include="*.js" --include="*.css" .
```
**Expected:** No matches in production files (matches in documentation are okay)

### 2. No Placeholder Domains
```bash
grep -r -i "example\.com\|yourdomain\|your-domain\|localhost" --include="*.html" --include="*.js" --include="*.txt" --include="*.xml" .
```
**Expected:** Only matches in documentation/template files, not in production code

### 3. No Secrets or Credentials
```bash
grep -r -i "password\|secret\|api.key\|api_key\|token\|credential\|private.key" --include="*.html" --include="*.js" --include="*.json" .
```
**Expected:** No matches

### 4. No Lorem Ipsum
```bash
grep -r -i "lorem ipsum\|dolor sit amet" --include="*.html" .
```
**Expected:** No matches

---

## File Existence Checks

### Required Files
```bash
# Check all required files exist
for file in index.html 404.html robots.txt sitemap.xml _headers assets/favicon.ico; do
  [ -f "$file" ] && echo "✓ $file" || echo "✗ $file MISSING"
done
```

### Security Files
```bash
# Check security files
for file in security.txt .well-known/security.txt LICENSE; do
  [ -f "$file" ] && echo "✓ $file" || echo "✗ $file MISSING"
done
```

### Favicon Set
```bash
# Check favicon files
for file in assets/favicon.ico assets/favicon-16x16.png assets/favicon-32x32.png assets/apple-touch-icon.png assets/android-chrome-192x192.png assets/android-chrome-512x512.png site.webmanifest; do
  [ -f "$file" ] && echo "✓ $file" || echo "✗ $file MISSING"
done
```

---

## Manual Verification Checklist

### Domain References
- [ ] All URLs point to your production domain (not `scqcs.com` or `example.com`)
- [ ] `sitemap.xml` uses your domain
- [ ] `robots.txt` sitemap reference uses your domain
- [ ] `security.txt` canonical URL uses your domain
- [ ] Open Graph `og:url` and `og:image` use your domain
- [ ] JSON-LD structured data uses your domain

### Internal Links
- [ ] All anchor links (`#section`) point to existing IDs
- [ ] No broken internal links
- [ ] Footer links work

### External Links
- [ ] All external links are valid (not 404)
- [ ] External links have `rel="noopener noreferrer"`
- [ ] External links have `target="_blank"` where appropriate

### Security.txt
- [ ] `Expires` date is in the future (recommend 1 year out)
- [ ] `Contact` information is correct
- [ ] `Canonical` URL matches where file is hosted

### Legal
- [ ] Privacy notice present if collecting any data
- [ ] Copyright year is current (use JavaScript for auto-update)
- [ ] License file present
- [ ] Disclaimer present if making security/legal claims

---

## Content Security Policy Check

Verify your CSP doesn't block needed resources:

```bash
# Extract CSP from _headers
grep -A1 "Content-Security-Policy" _headers
```

**Common CSP issues:**
- External fonts blocked → Add `https://fonts.googleapis.com` to `style-src`, `https://fonts.gstatic.com` to `font-src`
- Images from CDN blocked → Add CDN domain to `img-src`
- Inline styles broken → Ensure `'unsafe-inline'` in `style-src` (acceptable for styles)
- Scripts blocked → External JS should work with `'self'`, avoid inline scripts

---

## Post-Deploy Verification

After deploying, verify with these tools:

| Check | Tool |
|-------|------|
| Security Headers | https://securityheaders.com |
| SSL/TLS Grade | https://www.ssllabs.com/ssltest/ |
| CSP Evaluation | https://csp-evaluator.withgoogle.com |
| Structured Data | https://search.google.com/test/rich-results |
| Open Graph Preview | https://developers.facebook.com/tools/debug/ |
| Mobile Friendly | https://search.google.com/test/mobile-friendly |

---

## Quick Domain Replace

When using this repo as a template, replace the domain throughout:

**macOS/BSD:**
```bash
find . -type f \( -name "*.html" -o -name "*.txt" -o -name "*.xml" -o -name "*.json" -o -name "*.md" -o -name "_headers" -o -name "_redirects" \) \
  -exec sed -i '' 's/scqcs\.com/yourdomain.com/g' {} +
```

**Linux:**
```bash
find . -type f \( -name "*.html" -o -name "*.txt" -o -name "*.xml" -o -name "*.json" -o -name "*.md" -o -name "_headers" -o -name "_redirects" \) \
  -exec sed -i 's/scqcs\.com/yourdomain.com/g' {} +
```

Then manually update:
- [ ] `security.txt` contact information
- [ ] `humans.txt` organization details
- [ ] `llms.txt` project description
- [ ] JSON-LD organization name and details

---

## The "Ship It" Checklist

Final gut-check before deploying:

- [ ] Ran all command checks above (no unexpected output)
- [ ] All required files present
- [ ] Domain references updated
- [ ] security.txt not expired
- [ ] Tested on mobile
- [ ] Tested in incognito (no cached assets)
- [ ] Git repo has no uncommitted changes
- [ ] Ready to merge to main

---

## See Also

- [`SITE_SECURITY_CHECKLIST.md`](SITE_SECURITY_CHECKLIST.md) - Comprehensive security reference
- [`GETTING_STARTED.md`](GETTING_STARTED.md) - Deployment walkthrough
- [`_headers`](../_headers) - Security headers configuration

---

*Quick checks catch quick mistakes. Run this before every deploy.*
