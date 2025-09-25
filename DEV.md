This document is aimed at contributors and maintainers who want to develop, build, and test llumen locally or produce production artifacts.

> [!WARNING]
> The document is WIP, ~~I just list somethings here~~

Some Hints:

1. We used to have a nix contributor, nix config is unmaintained
2. We adopt `bits-ui` to improve animation performance, ~~sidebar performance sucks~~
3. previous nix contributor use `nushell` for script runner(`just`), maybe we should revert that?

TODO:

1. Release 0.1.0
  - Record a video
  - Check CI working
  - Repackage windows artifact, check if it's working
2. Fix mobile UI
  - `group-hover` hurt mobile UI, see copy button
  - refactor all frontend to use `bits-ui`
  - when user click `new chat` on mobile, close sidebar without props drilling~~(delay until refactoring)~~
3. Fix message sync
  - There is a small time gap in user messaging creation and halting
4. implement new feature
