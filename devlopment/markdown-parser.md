To support various LLM output text(LLM does not output standard markdown), we need to implement our custom parser.

> We does not support standard markdown, we support what LLM output.

## Compiler theory
I guess your(coding agent) training data include compiler theory!

In short, markdown is not a CFG, so LR/LL parser is impossible(even though you can delete some combination of production rule to make it CFG).

As the result, we implement a one-pass handwritten context-aware parser.

## Edge case to consider
- single newline for line breaks.
- dollar sign for latex, and often mixed with normal text
- rarely used `_`
