# Questions and answers procedure

1. Identify whether the question concerns current behavior, intended behavior,
   troubleshooting or a proposed design.
2. Locate entry points using the [architecture map](architecture.md). Read the
   relevant implementation and callers; consult documentation and tests as support.
3. Answer the question directly, then explain the relevant flow in plain language.
   Cite repository paths and symbols, with current line numbers when available.
4. Separate what code establishes, what tests cover, and what remains an inference.
   Do not describe proposed features as already implemented. If documentation and
   code disagree, identify the discrepancy.
5. For troubleshooting, gather only the minimum sanitized diagnostics needed.
   Distinguish development from installed builds, permissions, working directory
   and missing dependencies. Do not request private files or secrets.

Questions are read-only by default. Do not run cleanup, install software, modify
configuration or perform scans simply to explain how something works. A request
for a fix can move into implementation and the [QA workflow](qa.md).

If the repository cannot answer the question, say what evidence is missing rather
than inventing behavior. For external API facts, consult the relevant version's
official documentation and distinguish it from this application's implementation.

## Example requests

- “Explain how a plugin call travels from the GUI to the runner, citing the code.”
- “Where are quarantine restoration errors handled, and what does the user see?”
- “Which tests cover a disabled plugin, and what remains untested?”
