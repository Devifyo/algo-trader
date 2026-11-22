"""Pluggable AI providers behind a single interface. Nothing outside this
package (and the router) should import a concrete provider — the rest of the
platform only ever talks to the AIRouter."""
