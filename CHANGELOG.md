# 更新日志 / Changelog

本文件格式遵循 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.1.0/)，版本遵循语义化版本 2.0.0。
Format follows Keep a Changelog; versions follow Semantic Versioning 2.0.0.

## [auv-ui 0.1.0 · auv-i18n 0.1.0] - 2026-10-27

首次发布：AUV 系列共享 UI 与国际化基础库。
First release: shared UI and internationalization foundation libraries for the AUV series.

### 新增 / Added

- **fonts 模块**（`gui` feature）：内置 Sarasa Mono SC Regular（v1.0.43，SIL OFL 1.1，24.4MB）为 CJK 渲染基线；系统 8 候选链兜底（含 macOS STHeiti/Hiragino 回退）；`AuvUiCjkOrder` 入栈顺序参数（默认 `Primary`）。
- **EN**: fonts module (`gui` feature): embedded Sarasa Mono SC Regular (v1.0.43, SIL OFL 1.1, 24.4 MB) as the CJK rendering baseline, an 8-candidate system font fallback chain (incl. macOS STHeiti/Hiragino), and the `AuvUiCjkOrder` stacking control (default `Primary`).
- **term 模块**（常开）：终端能力探测——纯 std 零依赖、无 unsafe；`is_tty` / 颜色能力（None/Ansi16/Ansi256/Truecolor）/ Unicode 假定；纯函数核心全分支可测。
- **EN**: term module (always on): terminal capability detection in pure std with zero dependencies and no unsafe — `is_tty`, color capability (None/Ansi16/Ansi256/Truecolor), and Unicode assumption; pure-function core, all branches testable.
- **shell 模块**（`gui` feature）：`auv_ui_run_shell` 启动样板 + `auv_ui_native_options` 纯构造。
- **EN**: shell module (`gui` feature): `auv_ui_run_shell` bootstrap plus the pure `auv_ui_native_options` builder.
- **auv-i18n crate**（零 UI 依赖、无 unsafe、无异步运行时）：Fluent 机制层——
  - `auv_i18n_init`：启动注入 FTL 目录，zh-CN 源语言必须存在；装载问题收清单，永不 panic；
  - 语言探测：`AUV_LANG` > POSIX locale 变量（`LC_ALL`/`LC_MESSAGES`/`LANG`；`C`/`POSIX` 无倾向、继续下落）> 系统 UI 语言 > 缺省 zh-CN；
  - `auv_i18n_tr` / `auv_i18n_tr_fmt`：回退链 zh-CN→zh→en，缺键返回键本身；命名参数自动转 Fluent Number（复数选择器生效）；消费方非标准目录 tag（如 en-US/zh-Hans-CN）双向前缀兜底命中；
  - `auv_i18n_check_catalog`：目录校验（键字符集 / 重复键 / Fluent Term 条目检出）；
  - `terms` 术语模块：`term-` 域族级中性术语 zh-CN/en 双语目录 + 直查接口。
- **EN**: auv-i18n crate (zero UI dependencies, no unsafe, no async runtime) — the Fluent mechanism layer: `auv_i18n_init` injects FTL catalogs at startup (zh-CN required as the source language; load issues are collected, never panics); language detection follows `AUV_LANG` > POSIX locale vars (`C`/`POSIX` carry no opinion and fall through) > OS UI language > zh-CN default; `auv_i18n_tr` / `auv_i18n_tr_fmt` translate along the zh-CN→zh→en chain (a key missing everywhere yields the key itself; numeric args become Fluent Numbers so plural selectors work; non-standard consumer catalog tags such as en-US/zh-Hans-CN are matched via bidirectional prefix fallback); `auv_i18n_check_catalog` lints catalogs (key charset, duplicate keys, Fluent Term entries); the `terms` module ships family-neutral terminology for zh-CN/en with direct lookup.
- **feature 分闸**：`gui`/`tui` 默认关闭，消费方只付自己那份依赖。
- **EN**: feature gates: `gui`/`tui` are off by default — consumers pay only for what they enable.
- **安全与社区文档**：SECURITY.md（专用披露邮箱、72 小时响应承诺、协调披露、默认匿名致谢）；README 社区与贡献说明。
- **EN**: security and community documents: SECURITY.md (dedicated disclosure email, 72-hour acknowledgement commitment, coordinated disclosure, anonymous-by-default credit) and the community/contributing sections of the README.

### 验证状态 / Verification

- Windows 平台：测试与构建通过（43 项测试 + clippy `-D warnings` + fmt + release 零警告 + rustdoc 零警告）；macOS/Linux 随首个消费方应用发布时验证。
- **EN**: verified on Windows (43 tests + clippy `-D warnings` + fmt + warning-free release build + warning-free rustdoc); macOS/Linux verification follows with the first consumer application.
