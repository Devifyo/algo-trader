"""Provider interface + shared types. Every provider implements `complete()`
and reports token usage so the router can track cost. Providers raise
`ProviderError` with `retryable` set so the router knows whether to fail over."""

from __future__ import annotations

import abc
from dataclasses import dataclass


@dataclass
class Message:
    role: str          # "user" | "assistant"
    content: str


@dataclass
class CompletionResult:
    text: str
    input_tokens: int
    output_tokens: int
    model: str
    provider: str


class ProviderError(Exception):
    def __init__(self, message: str, *, retryable: bool, status: int | None = None):
        super().__init__(message)
        self.retryable = retryable      # True => try the next key/provider
        self.status = status


class AIProvider(abc.ABC):
    """One configured provider+key. Construct per (provider, api_key)."""

    name: str = "base"

    def __init__(self, api_key: str, base_url: str | None = None, timeout: float = 30.0):
        self.api_key = api_key
        self.base_url = base_url
        self.timeout = timeout

    @abc.abstractmethod
    async def complete(
        self,
        messages: list[Message],
        model: str,
        *,
        max_tokens: int = 1024,
        system: str | None = None,
    ) -> CompletionResult:
        ...
