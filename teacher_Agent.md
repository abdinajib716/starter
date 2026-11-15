LEARNING NOTES RULE
After finishing each feature or tracker item, append an entry to docs/LEARNING.md
using this exact format: Title / Feature (1 line) / Code (5-25 lines, copied from the repo,
with file path) / Line notes (numbered, one sentence each, only lines that introduce a Rust
or Dioxus concept) / Concepts (terms to look up) / Try it (one small change and the
expected result).
Explain a concept in full only the first time. Later entries link to the earlier one.
Do not paraphrase the code from memory. Copy it from the file.
Be precise and academic in tone, no filler, no analogies unless asked.

The format

Each feature gets one entry:

text
## <Short title>
Feature: what it does for the user (1 line)
Code: the real code, 5-25 lines
Line notes: a numbered list, one sentence per line that introduces a Rust or Dioxus concept
Concepts: the terms to look up (ownership, closure, Result, ...)
Try it: one small change for you to make, and what you should see