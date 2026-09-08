# Release preparation verification

Date: 2026-09-09. Environment: Apple Silicon Mac, Rust/Cargo 1.90.0, Xcode 26.5 (17F42).

| Check | Result |
| --- | --- |
| `./script/build_and_run.sh --test` | 38 unit tests and 1 Japanese text/IME integration test passed |
| `./script/build_and_run.sh --verify` | Release build, icon compilation, ad-hoc signature verification and app process launch passed |
| Native app accessibility inspection | Canvas toolbar and Input Monitoring permission dialog visible; permission was not granted during verification |
| README screenshot | Actual rectangle, text, and arrow operations captured from the running app; 820 × 460 PNG with only the bottom blank margin cropped to exclude the cursor; no generation or retouching |
| `./script/build_unsigned_installer.sh` | Apple Silicon installer and external SHA-256 generated |
| External installer checksum | Passed |
| Installer `--verify-only` | ZIP hash, code signature, bundle ID and arm64 architecture passed |
| Installation in `target/installer-check` | Passed without launching or modifying Applications folders |
| Reinstallation in the same isolated directory | Existing app preserved in dated backup directory |
| Installation without `--allow-unsigned` | Rejected before creating destination |
| Altered expected embedded ZIP checksum | Rejected with checksum mismatch |
| Shell syntax and unknown build option | Syntax passed; invalid option rejected with status 2 |
| README/document relative links | Passed |
| Publishable file inventory | No file over 50 MiB; build outputs and folder-local authentication excluded |
| Release download command | 8 local-fixture tests passed: valid verification, checksum mismatch, wrong filename, extra manifest line/field, download failure, explicit mode requirement, and exact parity with both README commands |
| Folder-local Git setup | Author, noreply email, GitHub credential helper, main branch and planned HTTPS origin configured in this checkout only |

Not verified: Windows execution, Intel Mac execution, macOS 14 hardware testing, end-to-end paste acceptance by another application, Developer ID signing, notarization, or GitHub publication. The local GitHub CLI still requires user authentication; no remote repository was created during these checks.

日本語: 既存テスト39件、Release版のビルド・署名・起動、インストーラの整合性・隔離フォルダへの配置・既存版バックアップ・破損拒否を確認しました。入力監視の権限は付与していません。Windows・Intel Mac・最小対応OSでの実機確認、公証、GitHub公開はこの検証の対象外です。
