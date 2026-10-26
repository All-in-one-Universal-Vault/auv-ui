// SPDX-License-Identifier: MulanPSL-2.0 OR Apache-2.0
//! 字体实测探针：开真实窗口验证内置 Sarasa Mono SC 的渲染效果。
//!
//! 用法 / Usage: `cargo run --example font_probe --features gui`
//!
//! 窗口内可切换 CjkOrder（Primary/Fallback）实时对比拉丁字形观感；
//! 显示字体安装报告（内置登记名 + 命中的系统兜底链路径）。

use auv_ui::fonts::{auv_ui_install_cjk_fonts, AuvUiCjkOrder, AuvUiFontReport};
use auv_ui::shell::{auv_ui_native_options, AuvUiShellSpec};
use eframe::egui;

struct Probe {
    order: AuvUiCjkOrder,
    report: AuvUiFontReport,
}

impl Probe {
    fn reinstall(&mut self, ctx: &egui::Context) {
        self.report = auv_ui_install_cjk_fonts(ctx, self.order);
    }
}

impl eframe::App for Probe {
    // eframe 0.36 形态：fn ui(ui, frame)。
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ui.horizontal(|ui| {
            if ui
                .selectable_label(
                    self.order == AuvUiCjkOrder::Primary,
                    "Primary（内置主字体）",
                )
                .clicked()
            {
                self.order = AuvUiCjkOrder::Primary;
                self.reinstall(ui.ctx());
            }
            if ui
                .selectable_label(
                    self.order == AuvUiCjkOrder::Fallback,
                    "Fallback（拉丁默认优先）",
                )
                .clicked()
            {
                self.order = AuvUiCjkOrder::Fallback;
                self.reinstall(ui.ctx());
            }
            ui.separator();
            ui.label(format!(
                "内置: {} | 系统兜底: {}",
                self.report.embedded,
                self.report
                    .system_fallback
                    .as_deref()
                    .unwrap_or("（未命中）")
            ));
        });
        ui.separator();

        ui.label(egui::RichText::new("字号梯度 / Size ramp").strong());
        for size in [13.0f32, 16.0, 20.0, 26.0] {
            ui.label(
                egui::RichText::new(format!(
                    "{size}px 密码库已解锁 · 自动锁定 09:42 · Argon2id 42%"
                ))
                .size(size),
            );
        }
        ui.separator();
        ui.label(egui::RichText::new("等宽对齐 / Monospace").strong());
        ui.label(
            egui::RichText::new(
                "用户  admin@example.com\n密码  ••••••••••••\n哈希  3f:a9:77:01:cc\n条目  42 条",
            )
            .family(egui::FontFamily::Monospace),
        );
        ui.separator();
        ui.label("破折号——全角 | en–dash 窄 | hyphen-minus 短");
        ui.label("易混字符 0 O o 8 B 1 l I | ~ 像素级区分");
        ui.label("生僻字 麤龗灪龖厵爨齉 𠮷");
        ui.label("繁体探测 漢字臺灣 · 日韩探测 日本語 한국어");
        ui.separator();
        ui.label(
            egui::RichText::new("粗体强调（字号/颜色承担）")
                .size(22.0)
                .strong(),
        );
    }
}

fn main() -> eframe::Result<()> {
    let spec = AuvUiShellSpec::new("auv-ui-font-probe", [1100.0, 720.0]);
    let order = spec.cjk_order;
    let options = auv_ui_native_options(&spec);
    eframe::run_native(
        "auv-ui-font-probe",
        options,
        Box::new(move |cc| {
            let report = auv_ui_install_cjk_fonts(&cc.egui_ctx, order);
            Ok(Box::new(Probe { order, report }))
        }),
    )
}
