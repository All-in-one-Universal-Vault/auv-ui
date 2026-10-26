// SPDX-License-Identifier: MulanPSL-2.0 OR Apache-2.0
//! auv-i18n：AUV 族国际化机制库（Fluent 方案）。
//!
//! i18n mechanism library for the AUV family, built on Project Fluent.
//!
//! 分层定位：本 crate 只含机制（L1）+ 术语 data 模块（L3，见 [`terms`]）；
//! 产品文案（L2）在各软件仓，FTL 目录由消费方在 [`auv_i18n_init`] 时注入。
//! 键约束：ASCII 语义键
//! `[a-zA-Z][a-zA-Z0-9_-]*`、连字符分层禁点号、命名参数 `{ $name }`、
//! 复数用 Fluent 选择器、回退链 zh-CN→zh→en、缺键返回键本身。
//!
//! 纪律：零 UI 依赖（不碰 egui/eframe/ratatui）、无异步运行时、无 unsafe、
//! 无网络、无写盘；缺省永不 panic、永不空串。
//!
//! This crate carries the mechanism (L1) plus the shared terminology data
//! module (L3, [`terms`]) only; product copy (L2) lives in each product repo
//! and is injected via [`auv_i18n_init`]. Fallback chain zh-CN→zh→en; a key
//! missing from the whole chain yields the key itself.

use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::OnceLock;
// Mutex 仅服务 debug 告警去重集；release 下不参与编译（unused import 兑底）。
#[cfg(debug_assertions)]
use std::sync::Mutex;

use fluent::concurrent::FluentBundle;
use fluent::{FluentArgs, FluentError, FluentResource, FluentValue};
use unic_langid::LanguageIdentifier;

pub mod terms;

/// 界面语言。
///
/// UI language.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AuvI18nLang {
    /// 简体中文（源语言；族内各软件源文案为中文）。
    ZhCn,
    /// English。
    En,
}

const AUV_I18N_LANG_ZH: u8 = 0;
const AUV_I18N_LANG_EN: u8 = 1;

/// 回退链基准（当前语言之后依序回落；zh-CN 为源语言）。
///
/// Fallback chain after the current language (zh-CN is the source language).
const AUV_I18N_FALLBACK_TAIL: [&str; 3] = ["zh-CN", "zh", "en"];

/// 语言探测环境变量（`AUV_LANG` 最高优先，其次 POSIX locale 变量）。
const AUV_I18N_ENV_PRIMARY: &str = "AUV_LANG";
const AUV_I18N_ENV_POSIX: [&str; 3] = ["LC_ALL", "LC_MESSAGES", "LANG"];

static AUV_I18N_LANG_NOW: AtomicU8 = AtomicU8::new(AUV_I18N_LANG_ZH);

/// 已装载的目录：规范化 locale tag → bundle（同 tag 多 resource 已并入）。
///
/// 用并发 memoizer 线（`concurrent::FluentBundle`，内部 Mutex）：非并发线
/// 的 memoizer 含 `Box<dyn Any>`（非 Send）进不了静态；并发线 Send + Sync，
/// 可直接驻 `OnceLock`（初始化后只读，无需外层锁）。
type Store = HashMap<String, FluentBundle<FluentResource>>;
static AUV_I18N_STORE: OnceLock<Store> = OnceLock::new();

#[cfg(debug_assertions)]
static AUV_I18N_WARNED: OnceLock<Mutex<HashSet<String>>> = OnceLock::new();

/// 语言 → 规范化 BCP-47 tag。
fn lang_tag(lang: AuvI18nLang) -> &'static str {
    match lang {
        AuvI18nLang::ZhCn => "zh-CN",
        AuvI18nLang::En => "en",
    }
}

/// 回退链：当前语言 tag 打头，随后 zh-CN→zh→en（去重保序）。
fn lang_chain(lang: AuvI18nLang) -> Vec<&'static str> {
    let mut chain = vec![lang_tag(lang)];
    for tag in AUV_I18N_FALLBACK_TAIL {
        if !chain.contains(&tag) {
            chain.push(tag);
        }
    }
    chain
}

/// 键字符集校验：Fluent 语法标识符 `[a-zA-Z][a-zA-Z0-9_-]*`。
///
/// 中文键与点号键在 Fluent 里物理非法（点号是属性访问符）。
fn is_valid_key(key: &str) -> bool {
    let mut chars = key.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

/// 装载目录（纯函数核心，便于测试）：tag 解析失败/FTL 解析错误/重复键
/// 全部收进问题清单，不 panic；FTL 部分损坏时仍保留可解析的条目。
fn build_store(catalogs: &[(&str, &str)]) -> (Store, Vec<String>) {
    let mut store = Store::new();
    let mut issues = Vec::new();
    for (tag, source) in catalogs {
        let langid: LanguageIdentifier = match tag.parse() {
            Ok(l) => l,
            Err(e) => {
                issues.push(format!("locale '{tag}': tag 解析失败（{e}）"));
                continue;
            }
        };
        let norm = langid.to_string();
        let res = match FluentResource::try_new((*source).to_string()) {
            Ok(res) => res,
            Err((res, errs)) => {
                for err in &errs {
                    issues.push(format!("locale '{tag}': FTL 解析错误: {err}"));
                }
                res
            }
        };
        let bundle = store
            .entry(norm)
            .or_insert_with(|| new_bundle(langid.clone()));
        if let Err(errs) = bundle.add_resource(res) {
            for err in &errs {
                issues.push(format!("locale '{tag}': resource 并入失败: {err:?}"));
            }
        }
    }
    if !store.contains_key("zh-CN") {
        issues.push("缺 zh-CN 目录（源语言必须存在）".to_string());
    }
    // 非标准 tag 目录（不在回退链内，如 "en-US"/"zh-Hans-CN"）只能经 lookup 的
    // 前缀匹配兜底命中，init 期给一条告警（debug 下 stderr 可见），避免静默落空。
    let mut tags: Vec<&str> = store.keys().map(String::as_str).collect();
    tags.sort_unstable();
    for tag in tags {
        if !AUV_I18N_FALLBACK_TAIL.contains(&tag) {
            issues.push(format!(
                "locale '{tag}': 不在回退链 {AUV_I18N_FALLBACK_TAIL:?} 内，运行时仅靠前缀匹配兜底"
            ));
        }
    }
    (store, issues)
}

fn new_bundle(langid: LanguageIdentifier) -> FluentBundle<FluentResource> {
    let mut bundle = FluentBundle::new_concurrent(vec![langid]);
    // 关掉 FSI/PDI 双向隔离标记（U+2068/U+2069，unicode-bidi lint 禁直接书写）：
    // 终端与部分 UI 栈会把它们渲染成豆腐块。
    bundle.set_use_isolating(false);
    bundle
}

/// 参数组装：`&str` 中能解析为数值的转 Fluent Number（复数选择器 `[one]`
/// 等类别只对 Number 生效）；Fluent 数字显示无分组分隔符（日期/数字不本地化的既定约束天然满足）。先按 i64 解析，再 f64，最后按字符串。
///
/// 陷阱：凡能 parse 成 i64/f64 的值一律转 Number 并被规范化——补零编号
/// `"007"`→`"7"`、`"09"`→`"9"`，科学计数法 `"1e5"`→`"100000"`。编号/编码类
/// 参数要保留原样的，须让值含非数字字符（如 `"No.007"`），或直接写进
/// FTL 文案不走参数；本 API 暂无强制字符串通道。
fn make_args<'a>(args: &[(&'a str, &'a str)]) -> FluentArgs<'a> {
    let mut out = FluentArgs::new();
    for (name, value) in args {
        let parsed = if let Ok(i) = value.parse::<i64>() {
            FluentValue::from(i)
        } else if let Ok(f) = value.parse::<f64>() {
            FluentValue::from(f)
        } else {
            FluentValue::from(*value)
        };
        out.set(*name, parsed);
    }
    out
}

/// 按 tag 找 bundle（双向前缀兜底）：
/// 1. 精确命中；
/// 2. 链 tag 按 "-" 逐级截短再试（"zh-Hans-CN"→"zh-Hans"→"zh"）；
/// 3. 反向兜消费方更长的目录 tag（链 "en" 兜目录 "en-US"；多候选取
///    字典序最小者，保证确定性）。
fn find_bundle<'a>(store: &'a Store, tag: &str) -> Option<&'a FluentBundle<FluentResource>> {
    if let Some(bundle) = store.get(tag) {
        return Some(bundle);
    }
    let mut head = tag;
    while let Some((shorter, _)) = head.rsplit_once('-') {
        if let Some(bundle) = store.get(shorter) {
            return Some(bundle);
        }
        head = shorter;
    }
    let mut extensions: Vec<&str> = store
        .keys()
        .map(String::as_str)
        .filter(|k| k.len() > tag.len() && k.starts_with(tag) && k.as_bytes()[tag.len()] == b'-')
        .collect();
    extensions.sort_unstable();
    extensions.first().and_then(|k| store.get(*k))
}

/// 查词（纯函数核心）：按回退链取第一个命中的 bundle 渲染；
/// Fluent 渲染错误（如缺参数）不 panic——渲染结果仍可用，但 debug 下
/// 去重 stderr 告警（避免残损文案静默流出）。
fn lookup(
    store: &Store,
    lang: AuvI18nLang,
    key: &str,
    args: Option<&FluentArgs>,
) -> Option<String> {
    for tag in lang_chain(lang) {
        if let Some(bundle) = find_bundle(store, tag) {
            if let Some(msg) = bundle.get_message(key) {
                if let Some(pattern) = msg.value() {
                    let mut errors: Vec<FluentError> = vec![];
                    let out = bundle.format_pattern(pattern, args, &mut errors);
                    if !errors.is_empty() {
                        warn_format_error(key);
                    }
                    return Some(out.into_owned());
                }
            }
        }
    }
    None
}

/// 调试告警（每「类别+键」去重一次，`debug_assertions` 下 stderr；release 静默）。
fn warn_once(kind: &str, key: &str) {
    #[cfg(debug_assertions)]
    {
        let set = AUV_I18N_WARNED.get_or_init(|| Mutex::new(HashSet::new()));
        if let Ok(mut set) = set.lock() {
            if set.insert(format!("{kind}:{key}")) {
                eprintln!("[auv-i18n] {kind} i18n key: {key}");
            }
        }
    }
    let _ = (kind, key);
}

/// 缺键调试警告（全链缺失，每键一次）。
fn warn_missing(key: &str) {
    warn_once("missing", key);
}

/// 渲染错误调试警告（如 `{ $name }` 缺参数，产出残损文案，每键一次）。
fn warn_format_error(key: &str) {
    warn_once("format-error", key);
}

/// 按环境探测语言（纯函数核心，全分支可测）：
/// `AUV_LANG` > `LC_ALL` > `LC_MESSAGES` > `LANG`（`C`/`POSIX`  locale 视为
/// 「无语言倾向」，不下结论、继续下落）> 系统 UI 语言（sys-locale）> 缺省 zh-CN。
fn detect_from(
    env: &dyn Fn(&str) -> Option<String>,
    sys: &dyn Fn() -> Option<String>,
) -> AuvI18nLang {
    if let Some(v) = env(AUV_I18N_ENV_PRIMARY) {
        let v = v.to_lowercase();
        if v.starts_with("en") {
            return AuvI18nLang::En;
        }
        if v.starts_with("zh") {
            return AuvI18nLang::ZhCn;
        }
        // 无法识别的 AUV_LANG 值不下结论，继续往下看 POSIX locale 变量。
    }
    for key in AUV_I18N_ENV_POSIX {
        if let Some(v) = env(key) {
            let v = v.to_lowercase();
            // 与 AUV_LANG 判定统一为 starts_with（比 contains 保守可预期：
            // "azh"/"x-zh" 这类值不再误判中文，落英文分支）。
            if v.starts_with("zh") {
                return AuvI18nLang::ZhCn;
            }
            // C / POSIX / C.UTF-8 这类值不携带语言倾向（Unix 工具链常见缺省），
            // 不据此判定英文，继续下落到系统 UI 语言。
            let no_opinion = v.is_empty()
                || v == "c"
                || v == "posix"
                || v.starts_with("c.")
                || v.starts_with("c@");
            if !no_opinion {
                return AuvI18nLang::En;
            }
        }
    }
    if let Some(v) = sys() {
        let v = v.to_lowercase();
        if v.starts_with("zh") {
            return AuvI18nLang::ZhCn;
        }
        if !v.is_empty() {
            return AuvI18nLang::En;
        }
    }
    AuvI18nLang::ZhCn
}

/// 初始化：注入 `(locale_tag, ftl_source)` 目录列表，启动时调一次。
///
/// zh-CN 必须存在（源语言）；重复调用忽略（目录以首次为准），装载问题
/// 在 `debug_assertions` 下输出到 stderr。永不 panic。
///
/// Initialize with `(locale_tag, ftl_source)` catalogs; call once at startup.
/// zh-CN is required (source language). Never panics.
pub fn auv_i18n_init(catalogs: &[(&str, &str)]) {
    let (store, issues) = build_store(catalogs);
    #[cfg(debug_assertions)]
    for issue in &issues {
        eprintln!("[auv-i18n] init: {issue}");
    }
    let _ = issues;
    if AUV_I18N_STORE.set(store).is_err() {
        #[cfg(debug_assertions)]
        eprintln!("[auv-i18n] init: 重复调用已忽略（目录以首次为准）");
    }
}

/// 探测语言：`AUV_LANG` > POSIX locale 变量（`C`/`POSIX` 无倾向、继续下落）
/// > 系统 UI 语言（sys-locale）> 缺省 zh-CN。
///
/// Detect language: `AUV_LANG` > POSIX locale env (`C`/`POSIX` carry no
/// opinion) > OS UI language (sys-locale) > default zh-CN.
pub fn auv_i18n_detect() -> AuvI18nLang {
    let env = |k: &str| std::env::var(k).ok();
    detect_from(&env, &sys_locale::get_locale)
}

/// 当前语言（缺省 zh-CN）。
///
/// Current language (default zh-CN).
pub fn auv_i18n_lang() -> AuvI18nLang {
    if AUV_I18N_LANG_NOW.load(Ordering::Relaxed) == AUV_I18N_LANG_EN {
        AuvI18nLang::En
    } else {
        AuvI18nLang::ZhCn
    }
}

/// 设置语言（立即生效；GUI 重排 / TUI 重绘由消费方触发）。
///
/// Set language, effective immediately.
pub fn auv_i18n_set_lang(lang: AuvI18nLang) {
    AUV_I18N_LANG_NOW.store(
        match lang {
            AuvI18nLang::ZhCn => AUV_I18N_LANG_ZH,
            AuvI18nLang::En => AUV_I18N_LANG_EN,
        },
        Ordering::Relaxed,
    );
}

/// 中英切换（ZhCn <-> En）。
///
/// Toggle between zh-CN and en.
pub fn auv_i18n_lang_toggle() {
    let next = match auv_i18n_lang() {
        AuvI18nLang::ZhCn => AuvI18nLang::En,
        AuvI18nLang::En => AuvI18nLang::ZhCn,
    };
    auv_i18n_set_lang(next);
}

/// 翻译一条键：回退链 zh-CN→zh→en（当前语言优先）；
/// 全链缺失返回键本身 + `debug_assertions` 下 stderr 警告（每键一次）。
/// 未初始化目录时同样返回键本身。永不 panic、永不空串。
///
/// Translate a key along the fallback chain; a key missing everywhere yields
/// the key itself (plus a deduplicated stderr warning in debug builds).
pub fn auv_i18n_tr(key: &str) -> String {
    auv_i18n_tr_fmt(key, &[])
}

/// 带命名参数的翻译：Fluent 命名参数 `{ $name }` 式替换。
///
/// 数字陷阱：能 parse 成 i64/f64 的参数值一律转 Fluent Number（复数选择器
/// 需要），但会被规范化——补零编号 `"007"`→`"7"`、`"09"`→`"9"`、科学计数法
/// `"1e5"`→`"100000"`。编号/编码类参数须含非数字字符（如 `"No.007"`）才能
/// 原样保留，或直接写进 FTL 文案不走参数。
///
/// Translate with Fluent named arguments (`{ $name}` placeables). Values that
/// parse as i64/f64 become Fluent Numbers (needed for plural selectors) and
/// are normalized (`"007"`→`"7"`, `"1e5"`→`"100000"`); keep codes/serial
/// numbers non-numeric (e.g. `"No.007"`) to preserve their exact form.
///
/// Fluent 渲染错误（如 `{ $name }` 缺参数）不 panic：产出残损文案，并在
/// `debug_assertions` 下去重 stderr 告警（`format-error` 前缀，与缺键的
/// `missing` 前缀区分）。
pub fn auv_i18n_tr_fmt(key: &str, args: &[(&str, &str)]) -> String {
    let render = |store: &Store| {
        if args.is_empty() {
            lookup(store, auv_i18n_lang(), key, None)
        } else {
            let args = make_args(args);
            lookup(store, auv_i18n_lang(), key, Some(&args))
        }
    };
    let result = AUV_I18N_STORE.get().and_then(render);
    match result {
        Some(text) => text,
        None => {
            warn_missing(key);
            key.to_string()
        }
    }
}

/// 目录校验助手：返回 FTL 解析错误 + 键字符集违规 + Term 条目 + 重复键
/// 清单（供消费方测试用；空 vec 表示干净）。locale tag 不可解析也计入清单。
///
/// Catalog lint: parse errors + key charset violations + unsupported Term
/// entries + duplicate keys (empty vec = clean), for consumer-side
/// consistency tests.
pub fn auv_i18n_check_catalog(locale: &str, ftl_source: &str) -> Vec<String> {
    let mut issues = Vec::new();
    if locale.parse::<LanguageIdentifier>().is_err() {
        issues.push(format!("locale '{locale}': tag 不可解析"));
    }
    let parsed = fluent_syntax::parser::parse_runtime(ftl_source);
    let (res, errs) = match parsed {
        Ok(res) => (res, Vec::new()),
        Err((res, errs)) => (res, errs),
    };
    for err in errs {
        // 字节切片计行号：pos 落在多字节字符中间也不会 panic（不切 &str）。
        let line = ftl_source.as_bytes()[..err.pos.start.min(ftl_source.len())]
            .iter()
            .filter(|b| **b == b'\n')
            .count()
            + 1;
        issues.push(format!(
            "locale '{locale}': 第 {line} 行解析错误: {}",
            err.kind
        ));
    }
    let mut seen: HashSet<&str> = HashSet::new();
    for entry in &res.body {
        match entry {
            fluent_syntax::ast::Entry::Message(msg) => {
                if !is_valid_key(msg.id.name) {
                    issues.push(format!(
                        "locale '{locale}': 键 '{}' 违反字符集 [a-zA-Z][a-zA-Z0-9_-]*",
                        msg.id.name
                    ));
                }
                if !seen.insert(msg.id.name) {
                    issues.push(format!(
                        "locale '{locale}': 重复键 '{}'（同目录出现多次，运行时先见为准）",
                        msg.id.name
                    ));
                }
            }
            fluent_syntax::ast::Entry::Term(term) => {
                issues.push(format!(
                    "locale '{locale}': Term 条目 '-{}' 不受支持（机制层只支持 Message，tr 查不到 Term，会显示裸键）",
                    term.id.name
                ));
            }
            _ => {}
        }
    }
    issues
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 测试目录：zh-CN 源语言 + en 全译 + zh 部分（仅一个键），
    /// 专供回退链/复数/参数各分支。
    const TEST_ZH_CN: &str = "\
msg-hello = 你好
msg-welcome = 欢迎，{ $name }！
vault-entry-count = { $count } 条条目
label-big = 大数 { $n }
";
    const TEST_ZH: &str = "\
msg-hello = 你好（zh）
";
    const TEST_EN: &str = "\
msg-hello = Hello
msg-welcome = Welcome, { $name }!
vault-entry-count = { $count ->
    [one] { $count } entry
   *[other] { $count } entries
}
";

    fn test_store() -> Store {
        let (store, issues) =
            build_store(&[("zh-CN", TEST_ZH_CN), ("zh", TEST_ZH), ("en", TEST_EN)]);
        assert!(issues.is_empty(), "测试目录必须干净: {issues:?}");
        store
    }

    #[test]
    fn fallback_prefers_current_language() {
        let store = test_store();
        assert_eq!(
            lookup(&store, AuvI18nLang::En, "msg-hello", None).as_deref(),
            Some("Hello")
        );
        assert_eq!(
            lookup(&store, AuvI18nLang::ZhCn, "msg-hello", None).as_deref(),
            Some("你好")
        );
    }

    #[test]
    fn fallback_chain_zh_cn_to_zh_to_en() {
        let store = test_store();
        // 当前 En 而 en 缺键 → 落 zh-CN（回退链第二站）；带参渲染验证链路可用。
        let args = make_args(&[("n", "7")]);
        assert_eq!(
            lookup(&store, AuvI18nLang::En, "label-big", Some(&args)).as_deref(),
            Some("大数 7")
        );
        // zh 目录覆盖 msg-hello：仅当当前语言为 ZhCn 时 zh-CN 先命中，
        // zh 层只在 zh-CN 缺键时才有机会——构造 zh-CN 缺键的目录验证。
        let (partial, _) = build_store(&[("zh-CN", "msg-other = 别\n"), ("zh", TEST_ZH)]);
        assert_eq!(
            lookup(&partial, AuvI18nLang::ZhCn, "msg-hello", None).as_deref(),
            Some("你好（zh）")
        );
        // 全无命中 → None（外壳返回键本身）。
        assert_eq!(lookup(&store, AuvI18nLang::ZhCn, "msg-absent", None), None);
    }

    #[test]
    fn zh_only_key_falls_to_zh_cn_not_en() {
        // vault-entry-count 中英都有：当前 ZhCn 命中 zh-CN 形态。
        let store = test_store();
        let args = make_args(&[("count", "3")]);
        assert_eq!(
            lookup(&store, AuvI18nLang::ZhCn, "vault-entry-count", Some(&args)).as_deref(),
            Some("3 条条目")
        );
    }

    #[test]
    fn tr_fmt_named_args_and_plural_selector() {
        let store = test_store();
        let args = make_args(&[("name", "小甲")]);
        assert_eq!(
            lookup(&store, AuvI18nLang::ZhCn, "msg-welcome", Some(&args)).as_deref(),
            Some("欢迎，小甲！")
        );
        // 复数选择器：数值参数走 Fluent Number，[one]/[other] 生效。
        let one = make_args(&[("count", "1")]);
        let many = make_args(&[("count", "2")]);
        assert_eq!(
            lookup(&store, AuvI18nLang::En, "vault-entry-count", Some(&one)).as_deref(),
            Some("1 entry")
        );
        assert_eq!(
            lookup(&store, AuvI18nLang::En, "vault-entry-count", Some(&many)).as_deref(),
            Some("2 entries")
        );
    }

    #[test]
    fn tr_fmt_unknown_arg_is_ignored() {
        let store = test_store();
        let args = make_args(&[("nope", "x")]);
        assert_eq!(
            lookup(&store, AuvI18nLang::En, "msg-hello", Some(&args)).as_deref(),
            Some("Hello")
        );
    }

    #[test]
    fn numbers_render_without_grouping() {
        // 日期/数字不本地化：Fluent Number 显示无分组分隔符。
        let store = test_store();
        let args = make_args(&[("n", "1234567")]);
        assert_eq!(
            lookup(&store, AuvI18nLang::ZhCn, "label-big", Some(&args)).as_deref(),
            Some("大数 1234567")
        );
    }

    #[test]
    fn check_catalog_accepts_clean_source() {
        let issues = auv_i18n_check_catalog("zh-CN", TEST_ZH_CN);
        assert!(issues.is_empty(), "{issues:?}");
    }

    #[test]
    fn check_catalog_rejects_chinese_and_dotted_keys() {
        let bad = "保存 = 存\ndialog.save-button = 保存\nok-key = 好\n";
        let issues = auv_i18n_check_catalog("zh-CN", bad);
        assert!(issues.len() >= 2, "中文键与点号键都必须被检出: {issues:?}");
        assert!(issues.iter().any(|i| i.contains("解析错误")));
    }

    #[test]
    fn check_catalog_flags_bad_locale_tag() {
        let issues = auv_i18n_check_catalog("zh_CN!!", "ok-key = 好\n");
        assert!(issues.iter().any(|i| i.contains("tag 不可解析")));
    }

    #[test]
    fn build_store_collects_issues_without_panic() {
        let (store, issues) = build_store(&[
            ("zh-CN", "ok-key = 好\n中文键 = 坏\n"),
            ("!!bad", "ok-key = x\n"),
            ("en", "dup-key = 1\ndup-key = 2\n"),
        ]);
        assert!(store.contains_key("zh-CN"));
        assert!(!store.contains_key("!!bad"));
        assert!(issues.iter().any(|i| i.contains("FTL 解析错误")));
        assert!(issues.iter().any(|i| i.contains("tag 解析失败")));
        assert!(issues.iter().any(|i| i.contains("并入失败")));
        // 部分损坏的 FTL 仍保留可解析条目。
        assert_eq!(
            lookup(&store, AuvI18nLang::ZhCn, "ok-key", None).as_deref(),
            Some("好")
        );
        // 重复键：first wins（add_resource 不覆盖）。
        assert_eq!(
            lookup(&store, AuvI18nLang::En, "dup-key", None).as_deref(),
            Some("1")
        );
    }

    #[test]
    fn build_store_requires_zh_cn() {
        let (_, issues) = build_store(&[("en", "ok-key = hi\n")]);
        assert!(issues.iter().any(|i| i.contains("缺 zh-CN")));
    }

    #[test]
    fn nonstandard_dir_tag_warns_but_hits_via_prefix() {
        // 消费方目录命名 "en-US"（不在回退链内）：init 期收告警，
        // lang=En 时链上 "en" 经前缀扩展兜底命中（不静默落空）。
        let (store, issues) = build_store(&[("zh-CN", TEST_ZH_CN), ("en-US", TEST_EN)]);
        assert!(
            issues.iter().any(|i| i.contains("不在回退链")),
            "非标准 tag 目录必须收告警: {issues:?}"
        );
        assert_eq!(
            lookup(&store, AuvI18nLang::En, "msg-hello", None).as_deref(),
            Some("Hello")
        );
        // 反向：目录 tag 是链 tag 的前缀截短（链 "zh-CN" 兜目录 "zh"）。
        let (store2, _) = build_store(&[("zh", TEST_ZH), ("en", TEST_EN)]);
        assert_eq!(
            lookup(&store2, AuvI18nLang::ZhCn, "msg-hello", None).as_deref(),
            Some("你好（zh）")
        );
    }

    #[test]
    fn check_catalog_flags_terms_and_duplicate_keys() {
        let src = "ok-key = 好\nok-key = 重复\n-brand-name = 某术语\n";
        let issues = auv_i18n_check_catalog("zh-CN", src);
        assert!(
            issues.iter().any(|i| i.contains("Term 条目")),
            "Term 条目必须被检出: {issues:?}"
        );
        assert!(
            issues
                .iter()
                .any(|i| i.contains("重复键") && i.contains("ok-key")),
            "重复键必须被检出: {issues:?}"
        );
    }

    #[test]
    fn check_catalog_line_count_survives_multibyte() {
        // 多字节字符 + 第二行解析错误：行号正确且零 panic（字节切片计行）。
        let src = "ok-key = 好\n中文字符 = 坏\n";
        let issues = auv_i18n_check_catalog("zh-CN", src);
        assert!(
            issues.iter().any(|i| i.contains("第 2 行")),
            "多字节后的行号必须正确: {issues:?}"
        );
    }

    #[test]
    #[cfg(debug_assertions)]
    fn format_error_warns_in_debug() {
        // 缺参渲染：{$n} 无参数 → 残损输出 + errors 非空 → debug 告警入去重集。
        let store = test_store();
        let out = lookup(&store, AuvI18nLang::ZhCn, "label-big", None);
        assert_eq!(out.as_deref(), Some("大数 {$n}"));
        let set = AUV_I18N_WARNED.get_or_init(|| Mutex::new(HashSet::new()));
        let set = set.lock().expect("warn 集合锁");
        assert!(
            set.contains("format-error:label-big"),
            "format-error 告警必须被触发: {set:?}"
        );
    }

    #[test]
    fn detect_posix_zh_only_at_start() {
        // 与 AUV_LANG 统一 starts_with：zh 出现在值中间不再判中文（落英文分支）。
        let env = env_of(&[("LANG", "azh_CN.UTF-8")]);
        assert_eq!(detect_from(&env, &sys_none), AuvI18nLang::En);
        // 值打头是 zh 仍判中文。
        let env = env_of(&[("LANG", "zh_Hans_CN")]);
        assert_eq!(detect_from(&env, &sys_none), AuvI18nLang::ZhCn);
    }

    fn env_of<'a>(pairs: &'a [(&'a str, &'a str)]) -> impl Fn(&str) -> Option<String> + 'a {
        move |k| {
            pairs
                .iter()
                .find(|(key, _)| *key == k)
                .map(|(_, v)| (*v).to_string())
        }
    }

    fn sys_none() -> Option<String> {
        None
    }

    #[test]
    fn detect_env_priority_auv_lang_wins() {
        let env = env_of(&[
            ("AUV_LANG", "en"),
            ("LC_ALL", "zh_CN.UTF-8"),
            ("LANG", "zh_CN"),
        ]);
        assert_eq!(detect_from(&env, &sys_none), AuvI18nLang::En);
    }

    #[test]
    fn detect_env_lc_all_beats_lang() {
        let env = env_of(&[("LC_ALL", "en_US.UTF-8"), ("LANG", "zh_CN.UTF-8")]);
        assert_eq!(detect_from(&env, &sys_none), AuvI18nLang::En);
        let env = env_of(&[("LC_MESSAGES", "zh_CN"), ("LANG", "en_US")]);
        assert_eq!(detect_from(&env, &sys_none), AuvI18nLang::ZhCn);
    }

    #[test]
    fn detect_env_unknown_auv_lang_falls_through() {
        let env = env_of(&[("AUV_LANG", "fr-FR"), ("LANG", "zh_CN.UTF-8")]);
        assert_eq!(detect_from(&env, &sys_none), AuvI18nLang::ZhCn);
    }

    #[test]
    fn detect_env_default_is_zh_cn() {
        assert_eq!(detect_from(&env_of(&[]), &sys_none), AuvI18nLang::ZhCn);
        // 空串不算语言设置。
        let env = env_of(&[("LC_ALL", ""), ("LANG", "en_US")]);
        assert_eq!(detect_from(&env, &sys_none), AuvI18nLang::En);
    }

    #[test]
    fn detect_c_locale_is_no_opinion_and_falls_to_sys() {
        // C/POSIX/C.UTF-8 不携带语言倾向：落到系统 UI 语言（macOS 常见场景）。
        fn sys_zh() -> Option<String> {
            Some("zh_CN".to_string())
        }
        let env = env_of(&[("LANG", "C.UTF-8")]);
        assert_eq!(detect_from(&env, &sys_zh), AuvI18nLang::ZhCn);
        let env = env_of(&[("LC_ALL", "POSIX")]);
        assert_eq!(detect_from(&env, &sys_zh), AuvI18nLang::ZhCn);
        // 系统语言非中文 → En。
        fn sys_en() -> Option<String> {
            Some("en-US".to_string())
        }
        assert_eq!(detect_from(&env_of(&[]), &sys_en), AuvI18nLang::En);
        // 系统语言探测不到 → 缺省 zh-CN。
        assert_eq!(detect_from(&env_of(&[]), &sys_none), AuvI18nLang::ZhCn);
        // POSIX 变量有明确语言时优先于系统探测。
        let env = env_of(&[("LANG", "en_US.UTF-8")]);
        assert_eq!(detect_from(&env, &sys_zh), AuvI18nLang::En);
    }

    #[test]
    fn key_charset_rules() {
        assert!(is_valid_key("vault-entry-count"));
        assert!(is_valid_key("a"));
        assert!(is_valid_key("a_b-c9"));
        assert!(!is_valid_key("9abc"));
        assert!(!is_valid_key(""));
        assert!(!is_valid_key("保存"));
        assert!(!is_valid_key("dialog.save-button"));
        assert!(!is_valid_key("-lead"));
    }

    /// 全局状态集成测试（独一份：全局 STORE/LANG 只允许这一个测试触碰，
    /// 避免并行测试互相污染）。
    #[test]
    fn global_api_end_to_end() {
        auv_i18n_init(&[("zh-CN", TEST_ZH_CN), ("en", TEST_EN)]);

        auv_i18n_set_lang(AuvI18nLang::ZhCn);
        assert_eq!(auv_i18n_lang(), AuvI18nLang::ZhCn);
        assert_eq!(auv_i18n_tr("msg-hello"), "你好");
        assert_eq!(
            auv_i18n_tr_fmt("msg-welcome", &[("name", "小甲")]),
            "欢迎，小甲！"
        );

        // toggle 往返。
        auv_i18n_lang_toggle();
        assert_eq!(auv_i18n_lang(), AuvI18nLang::En);
        assert_eq!(auv_i18n_tr("msg-hello"), "Hello");
        auv_i18n_lang_toggle();
        assert_eq!(auv_i18n_lang(), AuvI18nLang::ZhCn);

        // 回退链：en 无 label-big → zh-CN（缺参占位符按 Fluent 规范化形态原样输出）；
        // 全链缺失 → 键本身。
        auv_i18n_set_lang(AuvI18nLang::En);
        assert_eq!(auv_i18n_tr("label-big"), "大数 {$n}");
        assert_eq!(auv_i18n_tr("msg-totally-absent"), "msg-totally-absent");
        auv_i18n_set_lang(AuvI18nLang::ZhCn);
    }
}
