"""Exchange API Safety Layer. Evaluates a connected API key's permissions and
blocks dangerous configurations BEFORE a key is saved/used. The headline rule:
withdrawal-enabled keys are rejected. Returns a plain-language breakdown with
fix steps so beginners understand exactly what's wrong and how to fix it.

The exchange round-trip that fetches `permissions` is an adapter (e.g. Binance
GET /sapi/v1/account/apiRestrictions); this module evaluates the result."""

from __future__ import annotations

CHECKS = ["read_permission", "spot_permission", "margin_permission",
          "futures_permission", "trading_permission", "withdraw_permission"]


def evaluate(permissions: dict, *, ip_whitelisted: bool = False,
             reachable: bool = True, valid_key: bool = True) -> dict:
    """permissions: {check_name: bool}. Returns {safe, warnings[], breakdown, fix_steps}."""
    warnings: list[dict] = []
    fix_steps: list[str] = []

    if not valid_key:
        warnings.append(_w("invalid_api_key", "The API key or secret is invalid.",
                           "high"))
        fix_steps.append("Re-copy the API key and secret from the exchange; check for trailing spaces.")
    if not reachable:
        warnings.append(_w("exchange_unreachable", "Could not reach the exchange.",
                           "high"))
        fix_steps.append("Check the exchange status page and your server's outbound network.")

    # THE hard rule — withdrawals must be disabled.
    if permissions.get("withdraw_permission"):
        warnings.append(_w("withdraw_permission_detected",
                           "This key can withdraw funds. The platform never needs that and "
                           "refuses to store withdrawal-enabled keys.", "critical"))
        fix_steps.append("On the exchange, edit the API key and DISABLE the 'Enable Withdrawals' permission.")

    if not permissions.get("trading_permission") and not permissions.get("spot_permission"):
        warnings.append(_w("insufficient_permissions",
                           "This key cannot place trades, so bots can't operate.", "medium"))
        fix_steps.append("Enable 'Spot & Margin Trading' (or Futures) on the API key.")

    if not ip_whitelisted:
        warnings.append(_w("ip_not_whitelisted",
                           "The key is not restricted to this server's IP. Restricting it "
                           "greatly reduces risk if the key leaks.", "medium"))
        fix_steps.append("Add this server's outbound IP to the key's IP allow-list on the exchange.")

    # SAFE only if: valid, reachable, withdrawals OFF, and can trade.
    safe = (valid_key and reachable
            and not permissions.get("withdraw_permission")
            and (permissions.get("trading_permission") or permissions.get("spot_permission")))

    return {
        "safe": bool(safe),
        "recommendation": "ok_to_save" if safe else "fix_before_saving",
        "permission_breakdown": {c: bool(permissions.get(c, False)) for c in CHECKS},
        "warnings": warnings,
        "fix_steps": fix_steps,
    }


def _w(code: str, explanation: str, severity: str) -> dict:
    return {"code": code, "severity": severity, "explanation": explanation}
