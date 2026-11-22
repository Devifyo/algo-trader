# Kubernetes — deferred

The platform currently runs on a **single VPS via Docker Compose**, tuned for
latency and resource efficiency (see `docs/PERFORMANCE.md`). Kubernetes is **not
installed, configured, or depended on** at this stage.

This directory is a placeholder for the future multi-region rollout:

- `base/` — Kustomize base (Deployments/Services for each Rust binary split out,
  ai-engine, news-service, portfolio-service, dashboard; ConfigMaps/Secrets).
- `overlays/production/` — per-region overlays (`us-east-1`, `ap-south-1`) with a
  **NATS supercluster** across regions, per-region node pools, static egress
  Elastic IPs (SEBI), HPA, and Vault auto-unseal (KMS).

Do not wire this until the single-VPS deployment is outgrown.
