# Contributing to Rust Audio Converter

Thank you for your interest in contributing to **Rust Audio Converter**! This project is 100% free and open-source, aiming to provide an accessible, high-performance audio converter for everyone.

---

## Code of Conduct

Please maintain a respectful, welcoming, and collaborative environment for all contributors.

---

## Development Workflow

### 1. Setting Up Your Environment
- Ensure you have **Rust** installed (`rustup update stable`).
- Ensure you have a C compiler available (`gcc` / `g++` via MinGW on Windows, or build-essential on Linux).

### 2. Making Changes
- Work on a focused feature or fix in a separate Git branch:
  ```bash
  git checkout -b feature/my-cool-feature
  ```
- Keep the three crates decoupled:
  - `converter-core`: Audio processing, codecs, metadata, and format logic only. No GUI dependencies.
  - `converter-cli`: Command-line interface and argument handling.
  - `converter-app`: GUI layout, state management, and user interaction.

### 3. Testing & Verification
Before submitting a pull request, ensure tests pass and code is formatted:
```bash
cargo test
cargo clippy
cargo fmt -- --check
```

---

## Submitting Pull Requests

1. Commit your changes with clear, descriptive commit messages.
2. Push your branch to your GitHub fork.
3. Open a Pull Request against the `main` branch.
4. Describe the change and any format/codec implications in the PR description.
