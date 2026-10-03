*the design choice behind merging and multiplexing*

## What's merging?
Consider you have a gpt-oss-20B from groq, which is 5000 token per second, this cause two problem:
1. frontend buffer overflow: sol=>throttling by frontend
2. user experience degraded experience: sol=>batch token

So backend need to:
1. implement a puller-throttle stream
2. eagerly merge nearby token of the same type

## What's multiplexing
TLDR: we does not implement multiplexing, and does not plan to

Modern LLM support parallel toolcalls, which cause problem in frontend where we rely on previous message to determine which tool call backend is emitting. The best practice is introduction of context.

## Merging implementation
If we just merge tokens on the fly, that would cause fragmentation.

Consider two allocation for each token, 5000 TPS will incur 10000 allocation per second, which is double the rate to what a router can do(Yes, llumen can run on router!).

So we append to last string.
