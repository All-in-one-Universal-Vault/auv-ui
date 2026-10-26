// SPDX-License-Identifier: MulanPSL-2.0 OR Apache-2.0
//! CJK 字体：内置 Sarasa Mono SC（Regular）为渲染基线，系统候选链兜底。
//!
//! CJK fonts: embedded Sarasa Mono SC (Regular) as the rendering baseline,
//! with the system candidate chain as fallback.
//!
//! 内置字体：Sarasa Mono SC v1.0.43（SIL OFL 1.1，见 THIRD-PARTY-NOTICES.md）。

use std::sync::Arc;

/// 内置字体在 `FontDefinitions` 中的登记名。
pub const AUV_UI_FONT_EMBEDDED: &str = "auv-ui-sarasa-mono-sc";

/// 系统兜底字体在 `FontDefinitions` 中的登记名。
pub const AUV_UI_FONT_SYSTEM: &str = "auv-ui-cjk-system";

/// 内置字体字节（Sarasa Mono SC Regular，原样未修改）。
///
/// Embedded font bytes (unmodified Sarasa Mono SC Regular).
pub const AUV_UI_EMBEDDED_FONT_BYTES: &[u8] = include_bytes!("../assets/SarasaMonoSC-Regular.ttf");

/// 系统 CJK 兜底链（8 候选；以首个 GUI 消费者实测名单为基准）。
///
/// macOS 回退：PingFang 在部分系统缺失，
/// 须回退 STHeiti / Hiragino。
///
/// System CJK fallback chain (8 candidates).
pub const AUV_UI_SYSTEM_CJK_CANDIDATES: &[&str] = &[
    "C:\\Windows\\Fonts\\msyh.ttc",
    "C:\\Windows\\Fonts\\simhei.ttf",
    "/System/Library/Fonts/PingFang.ttc",
    "/System/Library/Fonts/STHeiti Light.ttc",
    "/System/Library/Fonts/STHeiti Medium.ttc",
    "/System/Library/Fonts/Hiragino Sans GB.ttc",
    "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
    "/usr/share/fonts/truetype/wqy/wqy-microhei.ttc",
];

/// CJK 字体入栈顺序（默认 `Primary`）。
///
/// CJK font stacking order.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum AuvUiCjkOrder {
    /// 内置字体作主字体：拉丁字母也用 Sarasa 的 Iosevka 部分，中英观感统一。
    ///
    /// Embedded font as primary: Latin glyphs come from Sarasa's Iosevka part
    /// too, giving a uniform mixed-script look.
    #[default]
    Primary,
    /// egui 默认拉丁字体优先，内置字体只作 CJK 回退（首个消费者的历史形态）。
    ///
    /// egui's default Latin fonts first; the embedded font only serves CJK.
    Fallback,
}

/// 字体安装报告（跨平台测试核验用）。
///
/// Font installation report (for cross-platform test runs).
#[derive(Clone, Debug)]
pub struct AuvUiFontReport {
    /// 内置字体登记名（恒定；内置字节随二进制分发，必然存在）。
    pub embedded: &'static str,
    /// 实际命中的系统兜底字体路径；`None` 表示系统链全部未命中（仅用内置）。
    ///
    /// Path of the system fallback font actually loaded; `None` means the
    /// whole system chain missed and only the embedded font is in use.
    pub system_fallback: Option<String>,
}

/// 安装 CJK 字体：内置 Sarasa 为基线，系统链兜底。
///
/// 在 `eframe` 的 `AppCreator` 里调用一次（`run_shell` 已代劳）。
///
/// Install CJK fonts: embedded Sarasa baseline + system chain fallback.
/// Call once inside the eframe `AppCreator` (`run_shell` does this for you).
pub fn auv_ui_install_cjk_fonts(ctx: &egui::Context, order: AuvUiCjkOrder) -> AuvUiFontReport {
    let (fonts, report) = auv_ui_font_definitions(order);
    ctx.set_fonts(fonts);
    report
}

/// 构造字体定义（纯数据构造，便于测试；一般不直接调用）。
///
/// Build the font definitions (pure data, kept separate for testability).
pub fn auv_ui_font_definitions(order: AuvUiCjkOrder) -> (egui::FontDefinitions, AuvUiFontReport) {
    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(
        AUV_UI_FONT_EMBEDDED.into(),
        Arc::new(egui::FontData::from_static(AUV_UI_EMBEDDED_FONT_BYTES)),
    );

    let system_hit = AUV_UI_SYSTEM_CJK_CANDIDATES
        .iter()
        .find_map(|p| std::fs::read(p).ok().map(|bytes| (*p, bytes)));
    let system_path = system_hit.as_ref().map(|(p, _)| (*p).to_string());
    if let Some((_, bytes)) = system_hit {
        fonts.font_data.insert(
            AUV_UI_FONT_SYSTEM.into(),
            Arc::new(egui::FontData::from_owned(bytes)),
        );
    }

    for family in [egui::FontFamily::Proportional, egui::FontFamily::Monospace] {
        if let Some(list) = fonts.families.get_mut(&family) {
            arrange_family(list, order, system_path.is_some());
        }
    }
    let report = AuvUiFontReport {
        embedded: AUV_UI_FONT_EMBEDDED,
        system_fallback: system_path,
    };
    (fonts, report)
}

/// 族内名单排布：兜底永远最后；内置字体的位置由 `order` 决定。
fn arrange_family(list: &mut Vec<String>, order: AuvUiCjkOrder, has_system: bool) {
    match order {
        AuvUiCjkOrder::Primary => list.insert(0, AUV_UI_FONT_EMBEDDED.into()),
        AuvUiCjkOrder::Fallback => list.push(AUV_UI_FONT_EMBEDDED.into()),
    }
    if has_system {
        list.push(AUV_UI_FONT_SYSTEM.into());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_font_parses_and_covers_cjk() {
        // 与 egui 光栅化栈同源的 ab_glyph 解析，校验资产完整性与覆盖。
        use ab_glyph::Font;
        let font = ab_glyph::FontVec::try_from_vec(AUV_UI_EMBEDDED_FONT_BYTES.to_vec())
            .expect("内置字体必须可被解析");
        for ch in ['密', '库', '龗', '麤', 'A', '0', '汉', '字'] {
            assert_ne!(font.glyph_id(ch).0, 0, "缺字形: {ch}");
        }
    }

    #[test]
    fn order_primary_puts_embedded_first() {
        let mut list = vec!["latin-default".to_string()];
        arrange_family(&mut list, AuvUiCjkOrder::Primary, true);
        assert_eq!(
            list,
            vec![AUV_UI_FONT_EMBEDDED, "latin-default", AUV_UI_FONT_SYSTEM]
        );
    }

    #[test]
    fn order_fallback_keeps_latin_first() {
        let mut list = vec!["latin-default".to_string()];
        arrange_family(&mut list, AuvUiCjkOrder::Fallback, true);
        assert_eq!(
            list,
            vec!["latin-default", AUV_UI_FONT_EMBEDDED, AUV_UI_FONT_SYSTEM]
        );
    }

    #[test]
    fn system_chain_absent_is_fine() {
        let mut list = vec!["latin-default".to_string()];
        arrange_family(&mut list, AuvUiCjkOrder::Primary, false);
        assert_eq!(list, vec![AUV_UI_FONT_EMBEDDED, "latin-default"]);
    }
}
