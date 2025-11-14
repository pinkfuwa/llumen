# Contributing to Llumen

> [!TIP]
> Llumen is small project, maybe you should contact maintainer directly.

Thank you for your interest in contributing to Llumen! We welcome contributions from everyone in the form of suggestions, bug reports, pull requests, and feedback. This document provides guidance to help you get started.

## Submitting Bug Reports and Feature Requests

If you encounter a bug or have a feature request, please open an issue in the main repository. When reporting a bug or asking for help, include enough details so that others can reproduce the behavior you are seeing. For tips on how to do this, see the guide on producing a [Minimal, Complete, and Verifiable example].

[Minimal, Complete, and Verifiable example]: https://stackoverflow.com/help/mcve

When making a feature request, please clearly describe the problem you want to solve, any ideas for how Llumen could support solving that problem, possible alternatives, and any disadvantages.

## Running the Test Suite

We encourage you to run the test suite locally before submitting a pull request. This helps catch issues early and makes the review process smoother.

### Backend (Rust)

```sh
cargo test
```

### Frontend (Svelte/TypeScript)

```sh
cd frontend
pnpm install
pnpm run test
```

If any tests fail, please address them before submitting your pull request.

## Code Style and Guidelines

- Prioritize code correctness and clarity.
- Avoid panics and always handle errors appropriately.
- Use full variable names and avoid abbreviations.
- For Rust, prefer propagating errors with `?` and avoid `unwrap()`.
- For Svelte, use runes for state management and TypeScript for all code.

## Pull Requests

- Fork the repository and create your branch from `main`.
- Make your changes in logical, self-contained commits.
- Add or update tests as appropriate.
- Document user-facing or architectural changes in `docs/user.md` or `docs/design.md`.
- Ensure your code passes all tests and adheres to the project guidelines.
- Submit a pull request with a clear description of your changes.

## Conduct

In all Llumen-related forums, we follow the [Rust Code of Conduct]. For escalation or moderation issues, please contact the project maintainers directly.

[Rust Code of Conduct]: https://www.rust-lang.org/policies/code-of-conduct

---

Thank you for helping make Llumen better!
