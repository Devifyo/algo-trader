"""DeepSeek provider. OpenAI-compatible API, so it reuses the OpenAI request
shape with DeepSeek's base URL and default model."""

from __future__ import annotations

from .base import CompletionResult, Message
from .openai_provider import OpenAIProvider

DEFAULT_MODEL = "deepseek-chat"
DEFAULT_BASE = "https://api.deepseek.com/v1/chat/completions"


class DeepSeekProvider(OpenAIProvider):
    name = "deepseek"

    def __init__(self, api_key: str, base_url: str | None = None, timeout: float = 30.0):
        super().__init__(api_key, base_url or DEFAULT_BASE, timeout)

    async def complete(self, messages: list[Message], model: str, *,
                       max_tokens: int = 1024, system: str | None = None) -> CompletionResult:
        result = await super().complete(messages, model or DEFAULT_MODEL,
                                        max_tokens=max_tokens, system=system)
        result.provider = self.name
        return result
