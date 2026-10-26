// SPDX-License-Identifier: MulanPSL-2.0 OR Apache-2.0
//! 终端能力探测：纯 std 零依赖、无 unsafe。
//!
//! Terminal capability detection: pure std, no dependencies, no unsafe.
//!
//! 设计说明：检测分两层——`auv_ui_classify_term` 是纯函数核心（注入环境
//! 变量读取器与 TTY 判定，全分支可测）；`auv_ui_detect_term` 读真实环境。
//! 终端尺寸不在本层（消费方若用 crossterm，`terminal::size()` 自取）。

use std::io::IsTerminal;

/// 终端颜色能力档位。
///
/// Terminal color capability.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AuvUiTermColor {
    /// 无颜色输出（非 TTY、TERM=dumb 或 NO_COLOR 生效）。
    None,
    /// ANSI 16 色。
    Ansi16,
    /// 256 色（TERM 含 256color，或现代 Windows 控制台保底）。
    Ansi256,
    /// 真彩色（COLORTERM=truecolor/24bit，或 Windows Terminal）。
    Truecolor,
}

/// 终端能力结论。
///
/// Detected terminal capabilities.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct AuvUiTermCaps {
    /// stdout 是否为终端（管道/重定向下为 false——测试环境即此分支）。
    pub is_tty: bool,
    /// 颜色能力。
    pub color: AuvUiTermColor,
    /// 是否假定 Unicode（宽度/符号）可用；TERM=dumb 时降级为 false。
    pub unicode: bool,
}

/// 纯函数核心：给定环境变量读取器与 TTY 判定，给出能力结论。
///
/// Pure core: classify from an env reader and a TTY verdict (fully testable).
pub fn auv_ui_classify_term(env: &dyn Fn(&str) -> Option<String>, is_tty: bool) -> AuvUiTermCaps {
    if !is_tty {
        return AuvUiTermCaps {
            is_tty: false,
            color: AuvUiTermColor::None,
            unicode: false,
        };
    }
    let term = env("TERM").unwrap_or_default();
    let dumb = term == "dumb";
    let color = if env("NO_COLOR").is_some() || dumb {
        AuvUiTermColor::None
    } else if env("COLORTERM")
        .is_some_and(|v| v.eq_ignore_ascii_case("truecolor") || v.eq_ignore_ascii_case("24bit"))
        || env("WT_SESSION").is_some()
    {
        AuvUiTermColor::Truecolor
    } else if term.contains("256color") {
        AuvUiTermColor::Ansi256
    } else if cfg!(windows) {
        // 现代 conhost/ConPTY 默认 VT 256 色可用。
        AuvUiTermColor::Ansi256
    } else {
        AuvUiTermColor::Ansi16
    };
    AuvUiTermCaps {
        is_tty: true,
        color,
        unicode: !dumb,
    }
}

/// 读真实环境探测当前终端。
///
/// Detect capabilities of the real current terminal.
pub fn auv_ui_detect_term() -> AuvUiTermCaps {
    let env = |k: &str| std::env::var(k).ok();
    auv_ui_classify_term(&env, std::io::stdout().is_terminal())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn env_of<'a>(pairs: &'a [(&'a str, &'a str)]) -> impl Fn(&str) -> Option<String> + 'a {
        move |k| {
            pairs
                .iter()
                .find(|(key, _)| *key == k)
                .map(|(_, v)| (*v).to_string())
        }
    }

    #[test]
    fn non_tty_yields_none() {
        let caps = auv_ui_classify_term(&env_of(&[("COLORTERM", "truecolor")]), false);
        assert_eq!(caps.color, AuvUiTermColor::None);
        assert!(!caps.is_tty && !caps.unicode);
    }

    #[test]
    fn no_color_wins() {
        let caps = auv_ui_classify_term(
            &env_of(&[("NO_COLOR", "1"), ("COLORTERM", "truecolor")]),
            true,
        );
        assert_eq!(caps.color, AuvUiTermColor::None);
    }

    #[test]
    fn colorterm_truecolor() {
        let caps = auv_ui_classify_term(&env_of(&[("COLORTERM", "truecolor")]), true);
        assert_eq!(caps.color, AuvUiTermColor::Truecolor);
    }

    #[test]
    fn windows_terminal_session_is_truecolor() {
        let caps = auv_ui_classify_term(&env_of(&[("WT_SESSION", "x")]), true);
        assert_eq!(caps.color, AuvUiTermColor::Truecolor);
    }

    #[test]
    fn term_256color() {
        let caps = auv_ui_classify_term(&env_of(&[("TERM", "xterm-256color")]), true);
        assert_eq!(caps.color, AuvUiTermColor::Ansi256);
    }

    #[test]
    fn term_dumb_degrades() {
        let caps = auv_ui_classify_term(&env_of(&[("TERM", "dumb")]), true);
        assert_eq!(caps.color, AuvUiTermColor::None);
        assert!(!caps.unicode);
    }

    #[test]
    fn platform_default_branch() {
        let caps = auv_ui_classify_term(&env_of(&[]), true);
        let expected = if cfg!(windows) {
            AuvUiTermColor::Ansi256
        } else {
            AuvUiTermColor::Ansi16
        };
        assert_eq!(caps.color, expected);
        assert!(caps.unicode);
    }

    #[test]
    fn detect_smoke_no_panic() {
        // 测试环境 stdout 被捕获（非 TTY）——正是非 TTY 分支的实跑覆盖。
        let caps = auv_ui_detect_term();
        let _ = format!("{caps:?}");
    }
}
