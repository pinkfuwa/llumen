This document is aimed at contributors and maintainers who want to develop, build, and test llumen locally or produce production artifacts.

> [!WARNING]
> The document is WIP, feel free to contact me if I question.

## Some Hints:

1. We used to have a nix contributor, nix config is unmaintained
2. We adopt `bits-ui` to improve animation performance
3. previous nix contributor use `nushell` for script runner(`just`), maybe we should revert that?

## TODO:

1. Add mobile screenshot
2. release 0.1.2
3. Fix flickering when SetInfiniteQueryData(state management layer)
4. (other branch): Add luau runtime and [code-mode](https://blog.cloudflare.com/code-mode/) (alternative for traditional tool calling)
  - working luau sandbox and log-replaying(each run is a edge in graph, keep latest tree up to **size**, and count heap size of Lua Table)
  - add file accessing api for lua
  - add curl tool
  - add SQL api with csv/parquet/arrow import(also, state and memory accounting is important)
  - add deep research (see `prompts/`)
5. Fix demo's halt problem by updating demo branch
