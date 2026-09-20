# Repository instructions

## Desktop releases

- Whenever the user asks to publish, release, or ship a desktop build, the release must include both supported platform builds:
  - Windows x64 as an NSIS setup executable, plus its updater artifact and signature.
  - macOS as a universal Apple Silicon/Intel DMG, plus its updater archive and signature.
- Use `.github/workflows/release-desktop.yml` for published releases so each platform is built on its native GitHub-hosted runner.
- Do not describe a desktop release as complete until both platform jobs succeed and the GitHub Release contains both Windows and macOS assets. If either build is blocked or fails, report the incomplete platform explicitly instead of silently publishing only the other one.
- Keep the application version synchronized in `package.json`, `package-lock.json`, `src-tauri/Cargo.toml`, `src-tauri/Cargo.lock`, and `src-tauri/tauri.conf.json` before creating a release tag.
- Never commit or print `TAURI_SIGNING_PRIVATE_KEY`; it belongs in GitHub Actions secrets.
