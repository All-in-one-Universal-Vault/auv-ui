# All-in-one Universal Vault UI（auv-ui）

[中文](#中文) · [English](#english)

## 中文

**展示全称**：auv-ui = **All-in-one Universal Vault UI**；auv-i18n = **All-in-one Universal Vault Internationalization**。仓库名、crate 名与版本 tag 使用工程标识 `auv-ui` / `auv-i18n`，不因展示全称改变。

**定位**：AUV 系列共享 UI 与国际化基础库。

**隐私与边界**：两个库均不主动发起网络连接（零网络出口）、不收集遥测或崩溃报告、不写任何用户数据；以源码形式分发，无内置更新通道。

### 模块

| 模块 | feature | 内容 |
|---|---|---|
| `fonts` | `gui` | CJK 字体：内置 Sarasa Mono SC（SIL OFL 1.1）为渲染基线 + 系统 8 候选链兜底 + `AuvUiCjkOrder` 入栈顺序参数 |
| `term` | （常开） | 终端能力探测：纯 std 零依赖、无 unsafe；`is_tty` / 颜色能力 / Unicode 假定 |
| `shell` | `gui` | egui 应用壳启动样板：`auv_ui_run_shell`（标题/尺寸/字体注入挂点） |
| `auv-i18n` | （独立 crate） | 国际化机制库（Fluent）：`auv_i18n_init` / `auv_i18n_tr` / `auv_i18n_tr_fmt` + 回退链 zh-CN→zh→en + `terms` 术语库模块；零 UI 依赖 |

### 用法

```toml
[dependencies]
auv-ui = { git = "https://gitee.com/All-in-one_Universal_Vault/auv-ui", tag = "auv-ui-v0.1.0", features = ["gui"] }
```

```rust
fn main() -> eframe::Result<()> {
    auv_ui::shell::auv_ui_run_shell(
        "my-app",
        auv_ui::shell::AuvUiShellSpec::new("我的应用", [1020.0, 700.0]),
        Box::new(|cc| Ok(Box::new(MyApp::default()))),
    )
}
```

### 用法（auv-i18n）

FTL 文案目录在消费方仓内，启动时注入；zh-CN 为源语言必须存在，术语库可随目录一并注入。语言来源：`AUV_LANG` 环境变量 > POSIX locale 变量（`LC_ALL`/`LC_MESSAGES`/`LANG`，`C`/`POSIX` 无倾向、继续下落）> 系统 UI 语言（sys-locale）> 缺省 zh-CN；运行时 `auv_i18n_set_lang` / `auv_i18n_lang_toggle` 立即生效。

```rust
use auv_i18n::{terms, AuvI18nLang};

auv_i18n::auv_i18n_init(&[
    ("zh-CN", include_str!("../locales/zh-CN.ftl")),
    ("en", include_str!("../locales/en.ftl")),
    terms::auv_i18n_terms_catalog(AuvI18nLang::ZhCn),
    terms::auv_i18n_terms_catalog(AuvI18nLang::En),
]);
auv_i18n::auv_i18n_set_lang(auv_i18n::auv_i18n_detect());

let title = auv_i18n::auv_i18n_tr("dialog-settings-title");
let count = auv_i18n::auv_i18n_tr_fmt("vault-entry-count", &[("count", "3")]);
```

### 构建与测试

`cargo test --all-features` + `cargo clippy --all-targets -- -D warnings` + `cargo fmt --check`。

字体探针：`cargo run --example font_probe --features gui`——开真实窗口对比 Primary/Fallback 两种入栈顺序的渲染观感（示例截图见 `docs/font-probe.png`）。

### 社区

欢迎用中文或英文提交 issue（bug 报告、功能建议、译文建议均可），回复跟随报告人语言。

行为准则（简短版）：尊重、就事论事；拒绝骚扰、歧视与广告。违规先警告后拉黑，争议由维护者裁量处理。

安全问题请勿开 issue——请走 [SECURITY.md](SECURITY.md) 的非公开报告渠道。

### 贡献

**当前暂缓接收代码 PR**；问题报告与改进建议照常欢迎，由维护者评估后在版本更新中吸收。政策调整时会在此更新。

翻译建议以 issue 提交（暂不接收 `.ftl` 补丁文件）。

共同维护者路径：有持续高质量 issue 与建议记录、认同本仓质量要求者，可逐步参与 issue 分类与回复；代码层协作随贡献政策开放后启动。

### 许可

代码：**MulanPSL-2.0 OR Apache-2.0** 双许可，可任选其一：

- [木兰宽松许可证，第 2 版](LICENSE-MulanPSL-2.0)
- [Apache License, Version 2.0](LICENSE-Apache-2.0)

内置字体 Sarasa Mono SC：SIL OFL 1.1（见 [THIRD-PARTY-NOTICES.md](THIRD-PARTY-NOTICES.md)，含许可全文）。

---

## English

**Display names**: auv-ui = **All-in-one Universal Vault UI**; auv-i18n = **All-in-one Universal Vault Internationalization**. Repository, crate, and version-tag identifiers remain `auv-ui` / `auv-i18n`.

**Position**: shared UI and internationalization foundation libraries for the AUV series.

**Privacy & boundaries**: neither library initiates network connections (zero network egress), collects telemetry or crash reports, or writes user data; distributed as source with no built-in update channel.

### Modules

| Module | Feature | Contents |
|---|---|---|
| `fonts` | `gui` | CJK fonts: embedded Sarasa Mono SC (SIL OFL 1.1) baseline + 8-candidate system fallback chain + `AuvUiCjkOrder` stacking control |
| `term` | (always on) | Terminal capability detection: pure std, zero deps, no unsafe; `is_tty` / color capability / Unicode assumption |
| `shell` | `gui` | egui application shell bootstrap: `auv_ui_run_shell` (title/size/font hook) |
| `auv-i18n` | (separate crate) | i18n mechanism library (Fluent): `auv_i18n_init` / `auv_i18n_tr` / `auv_i18n_tr_fmt` + zh-CN→zh→en fallback chain + `terms` terminology module; zero UI deps |

### Usage

```toml
[dependencies]
auv-ui = { git = "https://gitee.com/All-in-one_Universal_Vault/auv-ui", tag = "auv-ui-v0.1.0", features = ["gui"] }
```

```rust
fn main() -> eframe::Result<()> {
    auv_ui::shell::auv_ui_run_shell(
        "my-app",
        auv_ui::shell::AuvUiShellSpec::new("My App", [1020.0, 700.0]),
        Box::new(|cc| Ok(Box::new(MyApp::default()))),
    )
}
```

### Usage (auv-i18n)

FTL catalogs live in the consumer repo and are injected at startup; zh-CN is the required source language, and the terminology module can be injected alongside. Language source: `AUV_LANG` env var > POSIX locale vars (`C`/`POSIX` carry no opinion and fall through) > OS UI language (sys-locale) > zh-CN default; `auv_i18n_set_lang` / `auv_i18n_lang_toggle` take effect immediately at runtime.

```rust
use auv_i18n::{terms, AuvI18nLang};

auv_i18n::auv_i18n_init(&[
    ("zh-CN", include_str!("../locales/zh-CN.ftl")),
    ("en", include_str!("../locales/en.ftl")),
    terms::auv_i18n_terms_catalog(AuvI18nLang::ZhCn),
    terms::auv_i18n_terms_catalog(AuvI18nLang::En),
]);
auv_i18n::auv_i18n_set_lang(auv_i18n::auv_i18n_detect());

let title = auv_i18n::auv_i18n_tr("dialog-settings-title");
let count = auv_i18n::auv_i18n_tr_fmt("vault-entry-count", &[("count", "3")]);
```

### Build & test

`cargo test --all-features` + `cargo clippy --all-targets -- -D warnings` + `cargo fmt --check`.

Font probe: `cargo run --example font_probe --features gui` — a real window comparing the Primary/Fallback stacking orders (sample screenshot in `docs/font-probe.png`).

### Community

Issues are welcome in Chinese or English (bug reports, feature ideas, translation feedback); replies follow the reporter's language.

Code of conduct (short): be respectful and stay on topic; harassment, discrimination, and advertising are not tolerated. Violations lead to a warning, then a block; the maintainer's judgment settles disputes.

Please do not report security issues via public issues — use the private channel described in [SECURITY.md](SECURITY.md).

### Contributing

**Code PRs are currently paused**; problem reports and improvement suggestions are always welcome and are absorbed by the maintainer into version updates. This policy will be revised here when it changes.

Translation suggestions go through issues (no `.ftl` patch files for now).

Co-maintainer path: contributors with a sustained record of high-quality issues and suggestions who agree with this repository's quality bar may progressively help with issue triage; code-level collaboration opens together with the contribution policy.

### License

Code: dual-licensed under **MulanPSL-2.0 OR Apache-2.0**, at your option:

- [Mulan Permissive Software License, Version 2](LICENSE-MulanPSL-2.0)
- [Apache License, Version 2.0](LICENSE-Apache-2.0)

Bundled font Sarasa Mono SC: SIL OFL 1.1 (see [THIRD-PARTY-NOTICES.md](THIRD-PARTY-NOTICES.md) for the full text).
