// SPDX-License-Identifier: MulanPSL-2.0 OR Apache-2.0
//! auv-ui：AUV 族 UI 骨架共享库（UI skeleton shared library for the AUV family）。
//!
//! 收录纪律：只收「已被两个以上消费者真实复制过」的构件。本库永不依赖
//! 任何产品 crate；公开 API 不含任何产品名。
//!
//! Admission discipline: only components actually duplicated across two or
//! more consumers are admitted. Never depends on product crates; public API
//! carries no product names.
//!
//! # 模块 / Modules
//!
//! - `term`（始终编译 / always on）：终端能力探测，纯 std 零依赖。
//! - `fonts`（`gui` feature）：CJK 字体——内置 Sarasa Mono SC 为基线 +
//!   系统候选链兜底。
//! - `shell`（`gui` feature）：egui 应用壳启动样板。
//!
//! # Feature 分闸 / Feature gates
//!
//! `gui` 与 `tui` 默认关闭，消费方只付自己那份依赖。

#[cfg(feature = "gui")]
pub mod fonts;
#[cfg(feature = "gui")]
pub mod shell;
pub mod term;
