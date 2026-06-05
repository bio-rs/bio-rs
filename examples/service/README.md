# bio-rs Service Template

This template packages the released `biors` CLI as a local-first REST service.

```bash
docker build -f examples/service/Dockerfile \
  --build-arg BIORS_VERSION=0.54.0 \
  -t biors-service:0.54.0 .

docker run --rm -p 8787:8787 biors-service:0.54.0
```

Then validate the service:

```bash
curl -s http://127.0.0.1:8787/health
```

The service does not call external APIs, upload input data, or run model
inference. Add auth, TLS, ingress, logging, and retention controls before
exposing it outside a local or institution-controlled network.
