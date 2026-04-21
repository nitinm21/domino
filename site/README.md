# Domino site

Next.js 14 landing page for Domino. Deploy target: Vercel.

## Develop

```bash
cd site
npm install
npm run dev
# http://localhost:3000
```

## Deploy

```bash
cd site
vercel          # first time — links the directory
vercel --prod   # ships
```

Zero config: Vercel auto-detects Next.js. Security headers are set in `next.config.mjs`.
