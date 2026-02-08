# Getting Started: Deploy Your Secure Site in Minutes

This guide walks you through deploying a secure, privacy-respecting static site using this template and Netlify.

## Why Netlify?

**Netlify is a paid service** (with a generous free tier) that makes deploying secure static sites effortless:

- **Free tier includes**: HTTPS, global CDN, continuous deployment, custom domains
- **`_headers` support**: Our security headers work automatically—no server configuration
- **GitHub integration**: Push to GitHub, Netlify deploys automatically
- **Branch previews**: Test changes before they go live

**Alternatives**: Cloudflare Pages (also supports `_headers`), Vercel (uses `vercel.json`), GitHub Pages (limited header support).

---

## Step 1: Get the Template

### Option A: Fork on GitHub (Recommended)
1. Go to [github.com/kmay89/SCQCS](https://github.com/kmay89/SCQCS)
2. Click **Fork** in the top right
3. Choose your GitHub account
4. You now have your own copy to customize

### Option B: Clone Locally
```bash
git clone https://github.com/kmay89/SCQCS.git my-secure-site
cd my-secure-site
rm -rf .git
git init
git add .
git commit -m "Initial commit from SCQCS template"
```

---

## Step 2: Create a Netlify Account

1. Go to [netlify.com](https://www.netlify.com)
2. Click **Sign up**
3. Choose **Sign up with GitHub** (easiest integration)
4. Authorize Netlify to access your GitHub

---

## Step 3: Deploy Your Site

### From the Netlify Dashboard
1. Click **Add new site** → **Import an existing project**
2. Choose **GitHub**
3. Select your forked/cloned repository
4. Leave build settings empty (it's a static site)
5. Click **Deploy site**

### Your Site is Live!
Netlify gives you a random URL like `random-name-123.netlify.app`. You can:
- Use this URL immediately
- Add a custom domain later
- Change the Netlify subdomain in **Site settings** → **Domain management**

---

## Step 4: Customize for Your Project

### Files to Update

**Required changes** (search and replace `scqcs.com` with your domain):

| File | What to Change |
|------|---------------|
| `index.html` | Replace all content with yours |
| `robots.txt` | Change sitemap URL |
| `sitemap.xml` | Change all URLs |
| `security.txt` | Update contact email and URLs |
| `.well-known/security.txt` | Same as above |
| `site.webmanifest` | Change app name and colors |
| `llms.txt` | Rewrite for your project |
| `humans.txt` | Update team info |

**Optional customization**:

| File | When to Change |
|------|---------------|
| `_headers` | If you use external APIs, add them to CSP |
| `_redirects` | If you need URL redirects |
| `main.js` | If you want different interactions |

### Quick Domain Replacement

In your terminal:
```bash
# macOS/Linux
find . -type f \( -name "*.html" -o -name "*.xml" -o -name "*.txt" -o -name "*.json" \) \
  -exec sed -i '' 's/scqcs\.com/yourdomain.com/g' {} +

# Linux only
find . -type f \( -name "*.html" -o -name "*.xml" -o -name "*.txt" -o -name "*.json" \) \
  -exec sed -i 's/scqcs\.com/yourdomain.com/g' {} +
```

---

## Step 5: Add Your Custom Domain

### In Netlify Dashboard
1. Go to **Site settings** → **Domain management**
2. Click **Add custom domain**
3. Enter your domain (e.g., `yourdomain.com`)
4. Follow the DNS configuration instructions

### DNS Configuration
Add these records at your domain registrar:

**For apex domain (yourdomain.com)**:
```
Type: A
Name: @
Value: 75.2.60.5
```

**For www subdomain**:
```
Type: CNAME
Name: www
Value: your-site-name.netlify.app
```

### Enable HTTPS
1. Go to **Site settings** → **Domain management** → **HTTPS**
2. Click **Verify DNS configuration**
3. Click **Provision certificate**
4. HTTPS is now enabled with auto-renewal

---

## Step 6: Verify Security Headers

After deployment, test your security headers:

1. Go to [securityheaders.com](https://securityheaders.com)
2. Enter your site URL
3. You should see **A+** grade

### What You Get Out of the Box
- **X-Frame-Options**: Prevents clickjacking
- **Content-Security-Policy**: Blocks XSS and injection attacks
- **Strict-Transport-Security**: Forces HTTPS
- **Permissions-Policy**: Disables unnecessary browser features
- **Cross-Origin policies**: Isolates your site from others

---

## Step 7: Set Up Continuous Deployment

This is automatic! Every time you push to GitHub:
1. Netlify detects the change
2. Builds and deploys your site
3. Updates go live in seconds

### Branch Previews
1. Create a new branch: `git checkout -b feature/new-page`
2. Push to GitHub
3. Netlify creates a preview URL for that branch
4. Review changes before merging to main

---

## Common Customizations

### Adding External Resources to CSP

If you need to load resources from external domains (fonts, images, APIs), update `_headers`:

```
Content-Security-Policy: default-src 'self'; script-src 'self'; style-src 'self' https://fonts.googleapis.com https://your-api.com; font-src https://fonts.gstatic.com; img-src 'self' data: https://images.example.com; ...
```

### Adding Analytics (If You Must)

If you need analytics, add the domain to CSP and disclose it:
```
script-src 'self' https://analytics.example.com
```

Update `humans.txt` to disclose analytics usage.

### Adding a Contact Form

Use a service like Netlify Forms or Formspree:
1. Add `netlify` attribute to your form: `<form netlify>`
2. Submissions appear in Netlify dashboard
3. No backend code needed

---

## Troubleshooting

### "My styles/scripts aren't loading"
Check CSP in `_headers`. Your resources must be allowed:
- External CSS: Add domain to `style-src`
- External JS: Add domain to `script-src`
- External images: Add domain to `img-src`

### "Security headers test fails"
Verify `_headers` file is in root directory and properly formatted.

### "Site shows old content"
1. Clear Netlify cache: **Deploys** → **Trigger deploy** → **Clear cache and deploy site**
2. Hard refresh browser: `Ctrl+Shift+R` / `Cmd+Shift+R`

### "Custom domain not working"
1. DNS propagation takes up to 48 hours
2. Verify DNS records at your registrar
3. Check Netlify's DNS verification status

---

## What's Next?

- [ ] Review [SITE_SECURITY_CHECKLIST.md](SITE_SECURITY_CHECKLIST.md) before launch
- [ ] Update `security.txt` with your actual security contact
- [ ] Test your site at [securityheaders.com](https://securityheaders.com)
- [ ] Submit to [HSTS Preload](https://hstspreload.org) (optional, after confirming HTTPS works)
- [ ] Validate structured data at [Google Rich Results Test](https://search.google.com/test/rich-results)

---

## Support

This template is provided as-is under the MIT License. For help:
- Open an issue on [GitHub](https://github.com/kmay89/SCQCS/issues)
- Review the [security checklist](SITE_SECURITY_CHECKLIST.md)
- Check Netlify's [documentation](https://docs.netlify.com)

---

*Built with security and privacy as defaults, not afterthoughts.*
