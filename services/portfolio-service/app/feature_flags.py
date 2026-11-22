"""Feature flag resolver. Flags can be set at four scopes; the most specific
wins: user > tenant > plan > global. Used for controlled rollout and SaaS plan
gating, with instant disable."""

from __future__ import annotations

SCOPE_PRECEDENCE = ["user", "tenant", "plan", "global"]


def resolve(flag: str, context: dict, rows: list[dict]) -> bool:
    """rows: [{flag, scope, scope_id, enabled}]. context: {user_id, tenant_id, plan}.
    Returns the effective on/off for `flag`."""
    scope_value = {
        "user": str(context.get("user_id", "")),
        "tenant": str(context.get("tenant_id", "")),
        "plan": str(context.get("plan", "")),
        "global": "*",
    }
    candidates = [r for r in rows if r["flag"] == flag]
    for scope in SCOPE_PRECEDENCE:
        want = scope_value[scope]
        for r in candidates:
            if r["scope"] == scope and str(r.get("scope_id", "*")) in (want, "*"):
                return bool(r["enabled"])
    return False   # default-off for unknown flags (fail closed)
