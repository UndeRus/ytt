# GitHub Actions Workflows

This directory contains GitHub Actions workflows for CI/CD.

## Workflows

### `ci.yml` - Continuous Integration
Runs on every push and pull request:
- Runs `cargo clippy` for linting
- Runs `cargo test` for testing
- Checks code formatting with `cargo fmt`

### `build.yml` - Build and Release
Runs on:
- Push to `main` branch
- Pull requests to `main`
- Tags starting with `v*` (creates GitHub release)
- Manual trigger via `workflow_dispatch`

**Builds executables for:**
- Linux x86_64 (GNU libc)
- Linux x86_64 (musl - statically linked)
- macOS x86_64 (Intel)
- macOS arm64 (Apple Silicon)
- Windows x86_64

**Artifacts:**
- Each platform produces a compressed archive (`.tar.gz` for Unix, `.zip` for Windows)
- Artifacts are uploaded and available for download
- On tag push, creates a GitHub release with all artifacts and checksums

## Usage

### Manual Build Trigger

Go to Actions → Build → Run workflow → Run workflow

### Creating a Release

1. Create and push a tag:
   ```bash
   git tag v1.0.0
   git push origin v1.0.0
   ```

2. The workflow will automatically:
   - Build all platform binaries
   - Create a GitHub release
   - Upload all artifacts
   - Generate checksums

### Downloading Artifacts

- **From Actions**: Go to the workflow run → Download artifacts
- **From Releases**: Go to Releases page → Download from latest release
