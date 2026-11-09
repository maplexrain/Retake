# Contributing to Retake

Thank you for your interest in contributing to **Retake**! We welcome bug reports, feature proposals, and pull requests from everyone.

---

## Code of Conduct

By participating in this project, you agree to abide by our [Code of Conduct](CODE_OF_CONDUCT.md).

---

## How to Contribute

### 1. Reporting Bugs
- Check the [existing issues](https://github.com/maplexrain/Retake/issues) to avoid duplicates.
- Open a new issue with clear reproduction steps, environment details (Windows version, FFmpeg version), and expected vs. actual behavior.

### 2. Suggesting Enhancements
- Open an issue describing your proposed feature, use cases, and design ideas.
- Join the discussion before submitting large code changes to ensure alignment with project roadmap.

### 3. Submitting Pull Requests
1. Fork the repository and create your branch from `main`:
   ```bash
   git checkout -b feature/your-feature-name
   ```
2. Make your changes:
   - Ensure clean architecture: preserve the card-free flat UI style and compact height adaptivity.
   - Maintain documentation and code comments in English.
   - Ensure all user-facing strings are added to [`src/i18n.ts`](src/i18n.ts) with both `en` and `zh` translations.
3. Verify your build locally:
   ```bash
   npm run build
   cargo check --manifest-path src-tauri/Cargo.toml
   ```
4. Commit your changes with clear messages following Conventional Commits (`feat: ...`, `fix: ...`, `docs: ...`, `refactor: ...`).
5. Open a Pull Request against `main`.

---

## Development Setup

- **Prerequisites**:
  - Windows 10 2004+ or Windows 11
  - Node.js 18+ and npm
  - Rust stable (MSVC toolchain)
  - FFmpeg with `libx264` and `gdigrab`
- **Commands**:
  - `npm install` - Install frontend dependencies
  - `npm run dev` - Start dev Vite server & launch Tauri app
  - `npm run build` - Typecheck & build frontend bundle
  - `npm run tauri build` - Build release package (NSIS installer)

---

## License

By contributing to Retake, you agree that your contributions will be licensed under its [MIT License](LICENSE).
