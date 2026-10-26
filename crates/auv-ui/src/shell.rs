// SPDX-License-Identifier: MulanPSL-2.0 OR Apache-2.0
//! egui 应用壳：`run_native` 启动样板（标题/尺寸/字体注入挂点）。
//!
//! egui application shell: a thin `run_native` bootstrap
//! (title / size / font-injection hook).
//!
//! 设计边界：本模块只做 bootstrap——「`fn ui(ui, frame)`、
//! CentralPanel-only、克隆后进闭包」等 eframe 适配约定不进代码，
//! 由各消费方的使用文档维护。

use crate::fonts::{auv_ui_install_cjk_fonts, AuvUiCjkOrder};
use eframe::egui;

/// 壳规格：窗口标题/尺寸/字体顺序。
///
/// Shell spec: window title / size / font order.
#[derive(Clone, Debug)]
pub struct AuvUiShellSpec {
    /// 窗口标题（用户可见）。
    pub title: String,
    /// 初始内尺寸（逻辑像素）。
    pub inner_size: [f32; 2],
    /// 最小内尺寸（可选）。
    pub min_inner_size: Option<[f32; 2]>,
    /// CJK 字体入栈顺序。
    pub cjk_order: AuvUiCjkOrder,
}

impl AuvUiShellSpec {
    /// 最小构造：标题 + 尺寸，其余默认（`CjkOrder::Primary`）。
    pub fn new(title: impl Into<String>, inner_size: [f32; 2]) -> Self {
        Self {
            title: title.into(),
            inner_size,
            min_inner_size: None,
            cjk_order: AuvUiCjkOrder::Primary,
        }
    }

    /// 设置最小内尺寸（建造者）。
    pub fn with_min_inner_size(mut self, size: [f32; 2]) -> Self {
        self.min_inner_size = Some(size);
        self
    }

    /// 设置字体入栈顺序（建造者）。
    pub fn with_cjk_order(mut self, order: AuvUiCjkOrder) -> Self {
        self.cjk_order = order;
        self
    }
}

/// 由规格构造 `NativeOptions`（纯构造，可测试）。
///
/// Build `NativeOptions` from a spec (pure construction, testable).
pub fn auv_ui_native_options(spec: &AuvUiShellSpec) -> eframe::NativeOptions {
    let mut viewport = egui::ViewportBuilder::default()
        .with_inner_size(spec.inner_size)
        .with_title(spec.title.clone());
    if let Some(min) = spec.min_inner_size {
        viewport = viewport.with_min_inner_size(min);
    }
    eframe::NativeOptions {
        viewport,
        ..Default::default()
    }
}

/// 启动 egui 应用：先装 CJK 字体（内置 Sarasa 基线 + 系统链兜底），再创建 App。
///
/// Run the egui app: installs CJK fonts (embedded Sarasa baseline + system
/// chain fallback) before creating the app.
///
/// 字体安装报告在跨平台测试核验时可直接调 `fonts::auv_ui_install_cjk_fonts` 自取；
/// 壳内安装的报告默认丢弃（内置基线在，无警告必要）。
pub fn auv_ui_run_shell(
    app_name: &str,
    spec: AuvUiShellSpec,
    app_creator: eframe::AppCreator<'_>,
) -> eframe::Result<()> {
    let order = spec.cjk_order;
    let options = auv_ui_native_options(&spec);
    eframe::run_native(
        app_name,
        options,
        Box::new(move |cc| {
            let _ = auv_ui_install_cjk_fonts(&cc.egui_ctx, order);
            app_creator(cc)
        }),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spec_builders_chain() {
        let spec = AuvUiShellSpec::new("测试窗口", [1020.0, 700.0])
            .with_min_inner_size([960.0, 560.0])
            .with_cjk_order(AuvUiCjkOrder::Fallback);
        assert_eq!(spec.title, "测试窗口");
        assert_eq!(spec.min_inner_size, Some([960.0, 560.0]));
        assert_eq!(spec.cjk_order, AuvUiCjkOrder::Fallback);
    }

    #[test]
    fn native_options_construct() {
        let spec = AuvUiShellSpec::new("x", [100.0, 100.0]);
        let _ = auv_ui_native_options(&spec); // 能构造即为本层契约
    }
}
