// SPDX-License-Identifier: MulanPSL-2.0 OR Apache-2.0
//! 术语库（L3）：族级中性术语的 zh-CN / en 内嵌目录（data-only）。
//!
//! Shared terminology (L3): embedded zh-CN / en catalogs of family-neutral
//! terms (data only, no product copy).
//!
//! 纪律：
//! - 术语与消息同一套键约束，键固定 `term-` 域；
//! - 译文不得含产品名（file/pass 等字样红线）；
//! - 本模块只放「简单 `key = value` 单行条目」（无参数、无选择器），
//!   复杂形态请走消费方 L2 目录经 `auv_i18n_tr_fmt`。
//!
//! 两种用法：[`auv_i18n_terms_catalog`] 取出 `(locale_tag, ftl_source)`
//! 注入 [`crate::auv_i18n_init`] 走统一回退链；或 [`auv_i18n_term`]
//! 只读直查静态表（零初始化，用于 init 之前的早期场景）。

use std::collections::HashMap;
use std::sync::OnceLock;

use crate::AuvI18nLang;

/// 术语 zh-CN 目录（FTL；简单单行条目）。
pub const AUV_I18N_TERMS_ZH_CN: &str = "\
term-auto-lock = 自动锁定
term-recycle-bin = 回收站
term-vault = 保险库
term-clipboard = 剪贴板
";

/// 术语 en 目录（FTL；与 zh-CN 键集一致，由本模块测试保证）。
pub const AUV_I18N_TERMS_EN: &str = "\
term-auto-lock = Auto-lock
term-recycle-bin = Recycle Bin
term-vault = Vault
term-clipboard = Clipboard
";

/// 取某语言的术语 `(locale_tag, ftl_source)`，供注入 [`crate::auv_i18n_init`]。
///
/// The terms `(locale_tag, ftl_source)` pair for a language, ready to be
/// fed into [`crate::auv_i18n_init`].
pub fn auv_i18n_terms_catalog(lang: AuvI18nLang) -> (&'static str, &'static str) {
    match lang {
        AuvI18nLang::ZhCn => ("zh-CN", AUV_I18N_TERMS_ZH_CN),
        AuvI18nLang::En => ("en", AUV_I18N_TERMS_EN),
    }
}

/// 只读直查术语译文；未收录返回 `None`（不 fallback——调用方自行决定
/// 回落，通常接 [`crate::auv_i18n_tr`] 或键本身）。
///
/// Read-only term lookup; `None` when the key is not a known term.
pub fn auv_i18n_term(key: &str, lang: AuvI18nLang) -> Option<&'static str> {
    term_map(lang).get(key).copied()
}

/// 解析内嵌 FTL 为静态表（仅简单 `key = value` 行；常量全生命周期驻留，
/// 键值切片直接借用常量，零泄漏零拷贝）。
fn term_map(lang: AuvI18nLang) -> &'static HashMap<&'static str, &'static str> {
    static ZH_CN: OnceLock<HashMap<&'static str, &'static str>> = OnceLock::new();
    static EN: OnceLock<HashMap<&'static str, &'static str>> = OnceLock::new();
    let (cell, src) = match lang {
        AuvI18nLang::ZhCn => (&ZH_CN, AUV_I18N_TERMS_ZH_CN),
        AuvI18nLang::En => (&EN, AUV_I18N_TERMS_EN),
    };
    cell.get_or_init(|| parse_simple_ftl(src))
}

/// 简单 FTL 行解析：`key = value` 单行；空行与 `#` 注释行跳过。
fn parse_simple_ftl(src: &'static str) -> HashMap<&'static str, &'static str> {
    src.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .filter_map(|line| line.split_once('='))
        .map(|(k, v)| (k.trim(), v.trim()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auv_i18n_check_catalog;

    #[test]
    fn catalogs_are_clean_ftl() {
        assert!(auv_i18n_check_catalog("zh-CN", AUV_I18N_TERMS_ZH_CN).is_empty());
        assert!(auv_i18n_check_catalog("en", AUV_I18N_TERMS_EN).is_empty());
    }

    #[test]
    fn zh_cn_and_en_share_key_set() {
        let zh = term_map(AuvI18nLang::ZhCn);
        let en = term_map(AuvI18nLang::En);
        assert!(!zh.is_empty());
        let mut zh_keys: Vec<_> = zh.keys().collect();
        let mut en_keys: Vec<_> = en.keys().collect();
        zh_keys.sort();
        en_keys.sort();
        assert_eq!(zh_keys, en_keys, "术语两语言键集必须一致");
    }

    #[test]
    fn keys_obey_charset_and_term_domain() {
        for key in term_map(AuvI18nLang::ZhCn).keys() {
            assert!(key.starts_with("term-"), "术语键固定 term- 域: {key}");
            assert!(crate::is_valid_key(key), "术语键违反字符集: {key}");
        }
    }

    #[test]
    fn values_carry_no_product_names() {
        // 文案红线：译文不得含产品名（file/pass 字样，大小写不敏感）。
        for (lang, map) in [
            ("zh-CN", term_map(AuvI18nLang::ZhCn)),
            ("en", term_map(AuvI18nLang::En)),
        ] {
            for (key, value) in map {
                let v = value.to_lowercase();
                assert!(
                    !v.contains("file") && !v.contains("pass") && !v.contains("auv"),
                    "{lang} 术语 {key} 译文含产品名: {value}"
                );
            }
        }
    }

    #[test]
    fn term_lookup_hits_and_misses() {
        assert_eq!(
            auv_i18n_term("term-auto-lock", AuvI18nLang::ZhCn),
            Some("自动锁定")
        );
        assert_eq!(
            auv_i18n_term("term-auto-lock", AuvI18nLang::En),
            Some("Auto-lock")
        );
        assert_eq!(
            auv_i18n_term("term-recycle-bin", AuvI18nLang::En),
            Some("Recycle Bin")
        );
        assert_eq!(auv_i18n_term("term-nope", AuvI18nLang::ZhCn), None);
    }

    #[test]
    fn catalog_pairs_feed_init_shape() {
        let (tag, src) = auv_i18n_terms_catalog(AuvI18nLang::ZhCn);
        assert_eq!(tag, "zh-CN");
        assert!(src.contains("term-auto-lock"));
        let (tag, _) = auv_i18n_terms_catalog(AuvI18nLang::En);
        assert_eq!(tag, "en");
    }
}
