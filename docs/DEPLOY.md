# Deploying to one host (staging)

The staging site, `staging.stabileo.com` (landing + editor) and `admin.staging.stabileo.com`
(admin site), runs on one Hetzner VM. Production stays on GitHub Pages until the cut-over at
the end of this page. The model is scasaas's one-host deploy: the release is built inside Nix on
the CI runner, shipped with its signed Nix closure, and swapped into place over SSH.

| Workflow | Trigger |
|---|---|
| `deploy-staging.yml` | push to `klaus/staging-deploy`; manual |
| `rollback-staging.yml` | manual (typed `CONFIRM` for a migration rollback) |

Both share one concurrency group, so neither starts while the other holds the host.

## What runs where

```
host (Debian 13, infra/cloud-init/staging.yaml)
+-- Caddy                 TLS, both static sites, /api/* -> backend     (infra/caddy/Caddyfile)
+-- PostgreSQL            stabileo_staging, owner stabileo
+-- Nix (multi-user)      the release's runtime closure, imported on every deploy
+-- stabileo-backend      system unit, User=app                        (infra/systemd/stabileo-backend.service)
+-- /srv/stabileo/        owned by app
    +-- live/             the current release: backend/, frontends/{app,admin}/, caddy/, .env
    +-- releases/         the previous three, for rollback
    +-- gcroots/          keeps the Nix paths those releases need from being collected
    +-- incoming/         artifacts in flight
```

The backend listens on `127.0.0.1:4100` only. Each site serves its static build and forwards
`/api/*` to it, so the browser calls the API same-origin: no CORS configuration. The admin's
Google sign-in starts and ends on the admin origin, which is why `API_URL` is the admin URL.

## Host prerequisites

`infra/cloud-init/staging.yaml` does all of it at first boot. Fill its three placeholders
(deploy public key, database password, Nix public key) and paste it as the server's cloud
config. Open 22, 80 and 443 in the Hetzner Cloud Firewall. Check with `cloud-init status --wait`
and `/var/log/cloud-init-output.log`.

What it leaves behind, for reference:

- user `app`, SSH key = the public half of `DEPLOY_SSH_KEY`, sudo only for the exact
  `systemctl`, `caddy validate` and `install` commands the playbooks run;
- multi-user Determinate Nix with `extra-trusted-public-keys` = the public half of `NIX_SIGNING_KEY` in `/etc/nix/nix.custom.conf`.
  `app` is an ordinary Nix user, never a `trusted-user`, so the daemon admits only signed paths;
- PostgreSQL role `stabileo` (`CREATEDB`, `pg_signal_backend` for a later restore flow) and the
  database `stabileo_staging`;
- Caddy with `/etc/caddy/Caddyfile` and `/etc/caddy/stabileo.env` (the domains and root), from Caddy's own apt repository (Debian 13 ships 2.6.2, too old for this Caddyfile);
- the `stabileo-backend` unit, enabled, started by the first deploy.

Generate the keys once:

```bash
ssh-keygen -t ed25519 -N '' -C stabileo-staging-deploy -f deploy_ed25519
nix key generate-secret --key-name stabileo-staging-1 > nix-signing.sec
nix key convert-secret-to-public < nix-signing.sec     # the cloud-init placeholder
```

## Settings

All in the GitHub environment **`staging`**. Variables are plain, secrets are masked.

| Name | Kind | Value |
|---|---|---|
| `DEPLOY_HOST` | variable | the VM's IP or `staging.stabileo.com` |
| `DEPLOY_USER` | variable | `app` |
| `DEPLOY_KNOWN_HOSTS` | variable | `ssh-keyscan -t ed25519 <host>`, fingerprint checked against `ssh-keygen -lf /etc/ssh/ssh_host_ed25519_key.pub` on the host |
| `DEPLOY_SSH_KEY` | secret | the private key |
| `NIX_SIGNING_KEY` | secret | contents of `nix-signing.sec` |
| `SECRET_KEY_BASE`, `JWT_SECRET` | secret | `make release.secrets`, or `openssl rand -base64 64` |
| `DATABASE_URL` | secret | `ecto://stabileo:<password>@localhost/stabileo_staging` |
| `PHX_HOST` | variable | `staging.stabileo.com` |
| `API_URL` | variable | `https://admin.staging.stabileo.com` (the Google callback is built on it) |
| `ADMIN_FRONTEND_URL` | variable | `https://admin.staging.stabileo.com` |
| `TRUSTED_PROXIES` | variable | `127.0.0.1` (Caddy's `X-Forwarded-For` is believed) |
| `DEPLOY_ENV` | variable | `staging` |
| `GOOGLE_CLIENT_ID` | variable | OAuth client with redirect URI `https://admin.staging.stabileo.com/api/v1/admin/auth/google/callback` |
| `GOOGLE_CLIENT_SECRET` | secret | |
| `PORT`, `BIND_ADDRESS`, `POOL_SIZE`, `DATABASE_SSL`, `CORS_ALLOWED_ORIGINS`, `ADMIN_DOMAINS`, `*_TTL` | variable | optional; `backend/config/runtime.exs` is the reference |

Without the Google pair the admin site shows no sign-in button; add them and redeploy.

## What a deploy does

`deploy-staging.yml`:

1. `make backend.release.build`, `make web.build` (with `VITE_ACCOUNTS=1`), `make admin.build`,
   the prerender gate from the Pages deploy, `caddy validate`;
2. packages `backend/`, `frontends/app`, `frontends/admin`, `caddy/Caddyfile`, the store paths
   the release references (`.github/scripts/release-store-paths.sh`) and their closure, signed
   with `NIX_SIGNING_KEY`;
3. uploads the tarball to `/srv/stabileo/incoming/` and runs `infra/ansible/deploy-app.yml`,
   which imports the closure, pins it against GC, writes `.env`, boots the new runtime config
   once (`eval ":ok"`) while the old release still serves, installs the Caddyfile, stops the
   backend, migrates, swaps `live/`, starts, and waits for `/api/v1/health`.

## First deploy, then the first admin

1. Host up, both DNS names pointing at it, settings filled: Actions -> **Deploy staging** -> Run.
2. Allow the admins on the host (Google sign-in is limited to the allowlist plus the Workspace
   domains in `ADMIN_DOMAINS`):

```bash
ssh app@<host>
cd /srv/stabileo/live && set -a && . ./.env && set +a
RELEASE_DISTRIBUTION=none ./backend/bin/stabileo eval 'StabileoAuth.Release.allow_admin_email("you@lambdaclass.com")'
```

Logs: `journalctl -u stabileo-backend -f` (the `app` user is in `systemd-journal`) and
`journalctl -u caddy -f`.

## Rollback

Actions -> **Rollback staging** swaps `live/` for the newest entry in `releases/` and restarts
the backend; one step back per run. Tick **rollback_db** and type `CONFIRM` only if the release
being rolled back ran a bad migration.

## Production cut-over (later)

- drop `X-Robots-Tag` from `infra/caddy/Caddyfile` and set the production domains in
  `infra/caddy/stabileo.env`;
- move the push trigger to `main` behind a `DEPLOY_ON_PUSH` variable, as scasaas does, with a
  `production` environment;
- point `stabileo.com` at the host, remove `web/public/CNAME`, retire `deploy-gh-pages.yml`.
