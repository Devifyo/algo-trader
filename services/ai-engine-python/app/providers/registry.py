"""Provider registry — maps a provider name to its implementation class.
Adding DeepSeek / Grok / OpenRouter / Ollama / a custom provider is one line
here plus a small provider module; nothing else in the system changes."""

from __future__ import annotations

from .anthropic_provider import AnthropicProvider
from .base import AIProvider
from .deepseek_provider import DeepSeekProvider
from .gemini_provider import GeminiProvider
from .openai_provider import OpenAIProvider

_REGISTRY: dict[str, type[AIProvider]] = {
    "anthropic": AnthropicProvider,
    "openai": OpenAIProvider,
    "gemini": GeminiProvider,
    "deepseek": DeepSeekProvider,          # OpenAI-compatible
    # "openrouter": OpenRouterProvider,    # OpenAI-compatible — set base_url
    # "ollama": OllamaProvider,            # local; set base_url to the daemon
}


def build_provider(name: str, api_key: str, base_url: str | None, timeout: float) -> AIProvider:
    cls = _REGISTRY.get(name)
    if cls is None:
        raise ValueError(f"unknown provider: {name}")
    return cls(api_key=api_key, base_url=base_url, timeout=timeout)


def known_providers() -> list[str]:
    return list(_REGISTRY)
