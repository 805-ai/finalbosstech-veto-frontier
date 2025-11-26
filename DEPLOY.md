# Deployment Guide - FinalBoss Veto Frontier

Complete guide to deploy the production backend and frontend.

---

## Quick Deploy (Recommended)

### Option 1: Railway.app (Easiest)

**Backend + Database in one click:**

1. **Install Railway CLI:**
   ```bash
   npm install -g @railway/cli
   # Or: brew install railway
   ```

2. **Login and Deploy:**
   ```bash
   cd /c/Users/notga/finalbosstech-veto-frontier
   railway login
   railway init
   railway up
   ```

3. **Add PostgreSQL:**
   ```bash
   railway add --database postgres
   ```

4. **Set Environment Variables:**
   ```bash
   railway variables set DEFAULT_ORG_ID=00000000-0000-0000-0000-000000000001
   railway variables set RUST_LOG=info
   ```

5. **Get Deployment URL:**
   ```bash
   railway domain
   # Copy the URL (e.g., https://veto-frontier-backend.up.railway.app)
   ```

6. **Update Frontend Config:**
   - Edit `config.js`
   - Replace `API_BASE_URL` with your Railway URL

7. **Deploy Frontend to Vercel:**
   ```bash
   vercel --prod
   ```

**Done!** Your system is live.

---

## Option 2: Fly.io (Fast & Global)

**Backend:**

1. **Install Fly CLI:**
   ```bash
   powershell -Command "iwr https://fly.io/install.ps1 -useb | iex"  # Windows
   # Or: curl -L https://fly.io/install.sh | sh  # macOS/Linux
   ```

2. **Login:**
   ```bash
   fly auth login
   ```

3. **Deploy:**
   ```bash
   cd /c/Users/notga/finalbosstech-veto-frontier
   fly deploy
   ```

4. **Create Database:**
   ```bash
   fly postgres create --name veto-postgres
   fly postgres attach veto-postgres
   ```

5. **Run Migrations:**
   ```bash
   fly proxy 5432 -a veto-postgres &
   psql -h localhost -p 5432 -U postgres < database/schema.sql
   ```

6. **Get URL:**
   ```bash
   fly apps open
   # Note the URL
   ```

**Frontend:** Same as Railway (update config.js, deploy to Vercel)

---

## Option 3: Docker + Cloud VM (AWS/GCP/Azure)

**Build and Push:**

```bash
cd /c/Users/notga/finalbosstech-veto-frontier

# Build images
docker-compose build

# Tag for registry
docker tag veto-backend:latest ghcr.io/805-ai/veto-backend:latest

# Push
docker push ghcr.io/805-ai/veto-backend:latest

# Deploy to cloud VM
ssh your-vm
docker pull ghcr.io/805-ai/veto-backend:latest
docker-compose up -d
```

---

## Manual Step-by-Step Deployment

### Step 1: Prepare Repository

```bash
cd /c/Users/notga/finalbosstech-veto-frontier

# Check git status
git status

# Add all changes
git add .

# Commit
git commit -m "Production backend implementation complete

- PostgreSQL schema with ACID guarantees
- Rust backend with ED25519 signatures
- Query enforcement layer (403 on orphaned pointers)
- Cryptographic governance receipts
- Docker containerization
- CI/CD workflows
- Frontend wired to real API

Implements US 19/240,581 Claim 9 and US 63/920,993
Sub-8ms orphaning latency target
Production-ready for deployment"

# Push to GitHub
git push origin main
```

### Step 2: Deploy Backend to Railway

1. **Go to https://railway.app**
2. Click "New Project"
3. Select "Deploy from GitHub repo"
4. Choose `805-ai/finalbosstech-veto-frontier`
5. Railway detects `railway.toml` and `Dockerfile`
6. Click "Deploy"
7. Add PostgreSQL:
   - Click "+ New"
   - Select "Database" → "PostgreSQL"
   - Railway automatically sets `DATABASE_URL`
8. Run migrations:
   - Go to Settings → Variables
   - Note the `DATABASE_URL`
   - Locally: `psql $DATABASE_URL < database/schema.sql`
9. Get deployment URL:
   - Settings → Networking → Generate Domain
   - Copy URL (e.g., `veto-frontier-backend.up.railway.app`)

### Step 3: Update Frontend Configuration

```bash
cd /c/Users/notga/finalbosstech-veto-frontier

# Edit config.js
# Change line 17 from:
return 'https://veto-frontier-backend.up.railway.app';
# To your actual Railway URL

# Commit
git add config.js
git commit -m "Update API URL to production Railway backend"
git push
```

### Step 4: Deploy Frontend to Vercel

```bash
# Already connected to Vercel, just push
git push origin main

# Or manually deploy
vercel --prod
```

### Step 5: Verify Deployment

```bash
# Test backend health
curl https://your-railway-url.up.railway.app/health

# Expected response:
# {"status":"healthy","service":"veto-frontier-backend","version":"0.1.0"}

# Test frontend
open https://finalbosstech-veto-frontier.vercel.app
# Click "ZERO-MULTIPLIER VETO" button
# Should see real API calls in browser console
```

---

## Environment Variables

### Required for Backend:

```env
DATABASE_URL=postgresql://user:pass@host:5432/veto_frontier
HOST=0.0.0.0
PORT=8888
DEFAULT_ORG_ID=00000000-0000-0000-0000-000000000001
RUST_LOG=info
```

### Optional:

```env
CORS_ALLOWED_ORIGINS=https://finalbosstech-veto-frontier.vercel.app
SIGNING_PRIVATE_KEY=<ed25519_private_key>
SIGNING_PUBLIC_KEY=<ed25519_public_key>
```

**Note:** If `SIGNING_*` keys not set, backend generates new keypair on startup.

---

## Post-Deployment Checklist

### Backend:

- [ ] Health endpoint returns 200: `/health`
- [ ] CORS allows frontend origin
- [ ] Database connected (check logs)
- [ ] ED25519 keypair generated/loaded
- [ ] Create pointer works: `POST /api/pointer/create`
- [ ] Orphan pointer works: `POST /api/pointer/orphan`
- [ ] Resolve orphaned returns 403: `GET /api/pointer/resolve/:id`

### Frontend:

- [ ] Loads without errors
- [ ] API status indicator shows "Connected ✓"
- [ ] Veto button triggers real API calls
- [ ] Browser console shows actual receipt hashes
- [ ] Popup displays real signatures and timestamps
- [ ] Orphaned pointer resolution blocked (403 in logs)

### Database:

- [ ] All tables created
- [ ] Indexes present
- [ ] Foreign keys enforced
- [ ] Triggers active
- [ ] Demo org inserted

---

## Troubleshooting

### Backend Won't Start

**Check logs:**
```bash
railway logs  # Railway
fly logs      # Fly.io
docker-compose logs backend  # Docker
```

**Common issues:**
- DATABASE_URL not set → Set in Railway/Fly dashboard
- Port conflict → Railway/Fly handle this automatically
- Rust compilation error → Check Dockerfile, may need more memory

### Database Connection Failed

**Verify DATABASE_URL:**
```bash
echo $DATABASE_URL
# Should be: postgresql://user:pass@host:port/db
```

**Test connection:**
```bash
psql $DATABASE_URL -c "SELECT version();"
```

**Run migrations:**
```bash
psql $DATABASE_URL < database/schema.sql
```

### Frontend Can't Reach API

**Check CORS:**
- Backend logs should show `Origin: https://yourfrontend.com`
- If blocked, add to `CORS_ALLOWED_ORIGINS` env var

**Check API URL:**
- Open browser console on frontend
- Look for `[CONFIG] API Base URL: ...`
- Verify it matches your deployed backend URL

**Test directly:**
```bash
curl https://your-backend-url/health
```

### 403 Errors on All Requests

**Not a bug!** Orphaned pointers should return 403.

To test fresh pointer:
```bash
# Create pointer
curl -X POST https://your-backend/api/pointer/create \
  -H "Content-Type: application/json" \
  -d '{"subject_id":"test","content_hash":"hash123"}'

# Get pointer_id from response, then resolve
curl https://your-backend/api/pointer/resolve/{pointer_id}
# Should return 200

# Orphan it
curl -X POST https://your-backend/api/pointer/orphan \
  -H "Content-Type: application/json" \
  -d '{"pointer_id":"{pointer_id}","reason":"test"}'

# Try to resolve again
curl https://your-backend/api/pointer/resolve/{pointer_id}
# Should return 403 ✓ Enforcement works!
```

---

## Monitoring & Observability

### Railway Dashboard:

- **Metrics:** CPU, Memory, Network usage
- **Logs:** Real-time log streaming
- **Deployments:** History, rollbacks

### Custom Monitoring:

**Health checks:**
```bash
# Set up cron job
*/5 * * * * curl -f https://your-backend/health || echo "Backend down!"
```

**Database queries:**
```sql
-- Active vs orphaned pointers
SELECT status, COUNT(*) FROM pointers GROUP BY status;

-- Recent orphaning events
SELECT * FROM governance_receipts
WHERE operation = 'orphan'
ORDER BY timestamp DESC
LIMIT 10;

-- Receipt chain integrity
SELECT
    COUNT(*) as total_receipts,
    COUNT(DISTINCT pointer_id) as unique_pointers
FROM governance_receipts;
```

---

## Scaling

### Backend:

**Vertical (Railway/Fly):**
- Increase memory: 512MB → 1GB → 2GB
- Add vCPUs: 1 → 2 → 4

**Horizontal:**
```bash
railway scale --replicas 3  # Railway
fly scale count 3            # Fly.io
```

### Database:

**Railway PostgreSQL:**
- Automatically scales storage
- Upgrade plan for more connections

**Managed alternatives:**
- AWS RDS PostgreSQL
- Google Cloud SQL
- Azure Database for PostgreSQL

### CDN:

Frontend already on Vercel's edge network (global CDN).

---

## Cost Estimates

### Railway (Recommended):

- **Backend:** $5/month (512MB RAM)
- **Database:** $5/month (1GB storage)
- **Total:** ~$10/month

### Fly.io:

- **Backend:** $0 (free tier: 3x shared-cpu-1x, 256MB)
- **Database:** $0 (free tier: 1GB storage)
- **Total:** $0/month (hobby use)

### Vercel (Frontend):

- **Free tier:** Unlimited for personal/hobby projects
- **Pro:** $20/month (optional, for team features)

---

## Production Hardening

Before going to production with real users:

### Security:

- [ ] Generate dedicated ED25519 keypair
- [ ] Store in secrets manager (Railway/Fly secrets, AWS Secrets Manager)
- [ ] Enable HTTPS only (Railway/Fly do this automatically)
- [ ] Restrict CORS to specific origins
- [ ] Add rate limiting (optional: `tower-http` rate limit middleware)
- [ ] Add authentication (JWT tokens, API keys)

### Performance:

- [ ] Load test with `wrk` or `ab`: 10,000 req/s target
- [ ] Optimize database indexes based on query patterns
- [ ] Enable connection pooling (already configured: 10 connections)
- [ ] Add Redis caching for frequent queries (optional)

### Reliability:

- [ ] Set up health check monitoring (UptimeRobot, Pingdom)
- [ ] Configure auto-restart on failure (Railway/Fly default)
- [ ] Set up error alerting (Sentry, Rollbar)
- [ ] Database backups (Railway auto-backups, Fly `pg_dump`)

### Compliance:

- [ ] Review GDPR implications (pointer orphaning = right to erasure)
- [ ] Document audit trail access for regulators
- [ ] Encrypt data at rest (use `encrypted_payload` field)
- [ ] Log retention policy (90 days recommended)

---

## Rollback

If deployment fails:

### Railway:
```bash
railway rollback
```

### Fly.io:
```bash
fly releases
fly rollback <version>
```

### Vercel:
- Dashboard → Deployments → Previous deployment → Promote

### Manual:
```bash
git revert HEAD
git push
```

---

## Next Steps After Deployment

1. **Share URLs:**
   - Backend: https://your-backend.up.railway.app
   - Frontend: https://finalbosstech-veto-frontier.vercel.app

2. **Test end-to-end:**
   - Open frontend
   - Click veto button
   - Verify real API calls in console
   - Check receipt hashes and signatures

3. **Load test:**
   ```bash
   wrk -t4 -c100 -d30s https://your-backend/health
   ```

4. **Monitor:**
   - Railway dashboard
   - Browser console
   - Database queries

5. **Document:**
   - Update README with deployed URLs
   - Share with stakeholders
   - Add to patent portfolio documentation

---

## Support

**Issues?**
- Check logs: `railway logs` or `fly logs`
- Verify DATABASE_URL is set
- Test health endpoint
- Review CORS settings

**Questions?**
- Documentation: `SETUP.md`, `PROJECT_SUMMARY.md`
- API reference: `backend/README.md`

---

**Status:** Ready to deploy
**Time to deploy:** ~15 minutes (Railway) or ~30 minutes (manual)
**Cost:** $10/month (Railway) or $0/month (Fly.io free tier)
