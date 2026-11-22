"""Anthropic Claude provider — uses the official `anthropic` SDK.

Defaults to Claude Opus 4.8 (`claude-opus-4-8`); the router may request Sonnet
4.6 (`claude-sonnet-4-6`) or Haiku 4.5 (`claude-haiku-4-5`) per the cost/perf
mode. Trading analysis is a single request/response — no streaming needed."""

from __future__ import annotations

import anthropic

from .base import AIProvider, CompletionResult, Message, ProviderError

DEFAULT_MODEL = "claude-opus-4-8"


class AnthropicProvider(AIProvider):
    name = "anthropic"

    def __init__(self, api_key: str, base_url: str | None = None, timeout: float = 30.0):
        super().__init__(api_key, base_url, timeout)
        self._client = anthropic.AsyncAnthropic(api_key=api_key, timeout=timeout)

    async def complete(
        self,
        messages: list[Message],
        model: str,
        *,
        max_tokens: int = 1024,
        system: str | None = None,
    ) -> CompletionResult:
        model = model or DEFAULT_MODEL
        try:
            resp = await self._client.messages.create(
                model=model,
                max_tokens=max_tokens,
                system=system or anthropic.NOT_GIVEN,
                messages=[{"role": m.role, "content": m.content} for m in messages],
            )
        except anthropic.RateLimitError as e:
            raise ProviderError(str(e), retryable=True, status=429) from e
        except anthropic.APIStatusError as e:
            raise ProviderError(str(e), retryable=e.status_code >= 500, status=e.status_code) from e
        except anthropic.APIConnectionError as e:
            raise ProviderError(str(e), retryable=True) from e

        text = "".join(b.text for b in resp.content if b.type == "text")
        return CompletionResult(
            text=text,
            input_tokens=resp.usage.input_tokens,
            output_tokens=resp.usage.output_tokens,
            model=resp.model,
            provider=self.name,
        )
