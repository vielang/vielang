# VieLang Docker stack

Local infra needed by the app that Vercel/Node can't run itself — currently
just LiveKit + Redis for the video call flow. Everything runs from this
folder so the repo root stays clean.

## Files

| File                     | Purpose                                      |
| ------------------------ | -------------------------------------------- |
| `docker-compose.dev.yml` | Boots `livekit-server` + `redis`             |
| `livekit.yaml`           | LiveKit config (keys, ports, webhook target) |

The compose file references `./livekit.yaml` relative to itself, so keep
them side-by-side.

## Boot

From the repo root:

```bash
docker compose -f docker/docker-compose.dev.yml up -d
```

Or from inside `docker/`:

```bash
cd docker
docker compose -f docker-compose.dev.yml up -d
```

## Services

| Service   | Ports                                  | Notes                                                                                  |
| --------- | -------------------------------------- | -------------------------------------------------------------------------------------- |
| `livekit` | 7880 (WS), 7881 (TCP), 50000-50100/UDP | SFU + TURN; UDP range is small for Windows Docker friendliness. Prod uses 50000-60100. |
| `redis`   | 6379                                   | LiveKit signaling backend. No persistence (dev only).                                  |

## Dev credentials

Keys in `livekit.yaml` MUST match the app env:

```
LIVEKIT_API_KEY=devkey
LIVEKIT_API_SECRET=devsecret_at_least_32_chars_long_xxxxxxxx
NEXT_PUBLIC_LIVEKIT_WS_URL=ws://localhost:7880
```

## Verify

```bash
# LiveKit is up
curl -i http://localhost:7880/
# → HTTP/1.1 200 OK, body: "OK"

# Redis is up
docker exec -it $(docker ps --filter name=redis -q) redis-cli ping
# → PONG
```

## Webhook

`livekit.yaml` sends events to
`http://host.docker.internal:3000/api/livekit/webhook`. That resolves to the
host machine's Next.js dev server on both macOS and Windows Docker Desktop.
On Linux hosts, replace with your host IP or run docker with
`--add-host=host.docker.internal:host-gateway`.

## Teardown

```bash
docker compose -f docker/docker-compose.dev.yml down
```

Add `-v` to also drop the redis volume (harmless — there's no data).
