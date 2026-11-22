"""OpenAI provider via the REST Chat Completions API (httpx). Non-Claude
provider — kept dependency-light. Default model gpt-4o-mini for cost."""

from __future__ import annotations

import httpx

from .base import AIProvider, CompletionResult, Message, ProviderError

DEFAULT_MODEL = "gpt-4o-mini"
API_URL = "https://api.openai.com/v1/chat/completions"


class OpenAIProvider(AIProvider):
    name = "openai"

    async def complete(
        self,
        messages: list[Message],
        model: str,
        *,
        max_tokens: int = 1024,
        system: str | None = None,
    ) -> CompletionResult:
        model = model or DEFAULT_MODEL
        body_messages = []
        if system:
            body_messages.append({"role": "system", "content": system})
        body_messages += [{"role": m.role, "content": m.content} for m in messages]

        url = (self.base_url or API_URL)
        try:
            async with httpx.AsyncClient(timeout=self.timeout) as client:
                r = await client.post(
                    url,
                    headers={"Authorization": f"Bearer {self.api_key}"},
                    json={"model": model, "max_tokens": max_tokens, "messages": body_messages},
                )
            if r.status_code >= 400:
                raise ProviderError(r.text, retryable=r.status_code in (429, 500, 502, 503, 529),
                                    status=r.status_code)
            data = r.json()
        except httpx.HTTPError as e:
            raise ProviderError(str(e), retryable=True) from e

        usage = data.get("usage", {})
        text = data["choices"][0]["message"]["content"]
        return CompletionResult(
            text=text,
            input_tokens=usage.get("prompt_tokens", 0),
            output_tokens=usage.get("completion_tokens", 0),
            model=data.get("model", model),
            provider=self.name,
        )
