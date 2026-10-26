# 第三方开源组件声明（Third-Party Notices）

本仓（auv-ui / auv-i18n）使用以下开源组件，均与本作品 **MulanPSL-2.0 OR Apache-2.0** 双许可兼容。按各许可要求，分发时保留版权与许可声明；各组件完整许可文本见其上游仓库（内置字体的 OFL 全文随本文件附后）。

## 直接依赖 / Direct dependencies

| 组件 | 用途 | 许可 |
|---|---|---|
| eframe/egui | GUI 框架（`gui` feature；0.36.2） | MIT OR Apache-2.0 |
| ab_glyph | 仅测试/示例用：内置字体资产完整性校验（dev-dependency） | Apache-2.0 |
| fluent（含 fluent-bundle） | auv-i18n：FTL 目录装载与消息渲染（0.17 线） | Apache-2.0 OR MIT |
| fluent-syntax | auv-i18n：目录校验直用 AST（0.12 线） | Apache-2.0 OR MIT |
| unic-langid | auv-i18n：locale tag 解析与规范化（0.9 线） | MIT OR Apache-2.0 |
| sys-locale | auv-i18n：系统 UI 语言探测（0.3 线） | MIT OR Apache-2.0 |

## 内置资源 / Bundled assets

| 资源 | 用途 | 许可 |
|---|---|---|
| Sarasa Mono SC Regular（更纱黑体等宽简体，v1.0.43，`crates/auv-ui/assets/SarasaMonoSC-Regular.ttf`，原样未修改） | 内置 CJK 渲染基线字体 | SIL Open Font License 1.1（全文见下） |

资产核验 / Asset verification：`SarasaMonoSC-Regular.ttf` — 25,612,524 字节 — SHA-256 `f0db6e00d3f9ffc3643430483410fd463758d8da93204f1ed1d30278afc2db28`，提取自通过官方 SHA-256 校验的 [Sarasa-Gothic v1.0.43 发布归档](https://github.com/be5invis/Sarasa-Gothic/releases/tag/v1.0.43)，与归档内同名文件逐字节一致。

Sarasa 版权行 / Sarasa copyright notice:

> Copyright (c) 2015-2025, Renzhi Li (aka. Belleve Invis, belleve@typeof.net).
> Portions Copyright (c) 2016 The Inter Project Authors.
> Portions Copyright (c) 2014-2021 Adobe Systems Incorporated (http://www.adobe.com/), with Reserved Font Name 'Source'.
> Portions Copyright (c) 2012 Google Inc.

本库对该字体**未做任何修改**（原样分发）；若将来产生衍生版本（如子集化），衍生品不得使用保留字体名 'Source'。
The font is distributed unmodified; any future derivative (e.g. subsetted) must not use the Reserved Font Name 'Source'.

## SIL OPEN FONT LICENSE（Version 1.1）全文

```
Preamble
----------

The goals of the Open Font License (OFL) are to stimulate worldwide
development of collaborative font projects, to support the font creation
efforts of academic and linguistic communities, and to provide a free and
open framework in which fonts may be shared and improved in partnership
with others.

The OFL allows the licensed fonts to be used, studied, modified and
redistributed freely as long as they are not sold by themselves. The
fonts, including any derivative works, can be bundled, embedded,
redistributed and/or sold with any software provided that any reserved
names are not used by derivative works. The fonts and derivatives,
however, cannot be released under any other type of license. The
requirement for fonts to remain under this license does not apply
to any document created using the fonts or their derivatives.

Definitions
-------------

"Font Software" refers to the set of files released by the Copyright
Holder(s) under this license and clearly marked as such. This may
include source files, build scripts and documentation.

"Reserved Font Name" refers to any names specified as such after the
copyright statement(s).

"Original Version" refers to the collection of Font Software components as
distributed by the Copyright Holder(s).

"Modified Version" refers to any derivative made by adding to, deleting
from, or substituting -- in part or in whole -- any of the components of
the Original Version, by changing formats or by porting the Font Software to
a new environment.

"Author" refers to any designer, engineer, programmer, technical
writer or other personnel who contributed to the Font Software.

Permission & Conditions
------------------------

Permission is hereby granted, free of charge, to any person obtaining
a copy of the Font Software, to use, study, copy, merge, embed, modify,
redistribute, and sell modified and unmodified copies of the Font
Software, subject to the following conditions:

1. Neither the Font Software nor any of its individual components,
   in Original or Modified Versions, may be sold by itself.

2. Original or Modified Versions of the Font Software may be bundled,
   redistributed and/or sold with any software, provided that each copy
   contains the above copyright notice and this license. These can be
   included either as stand-alone text files, human-readable headers or
   in the appropriate machine-readable metadata fields within text or
   binary files as long as those fields can be easily viewed by the user.

3. No Modified Version of the Font Software may use the Reserved Font
   Name(s) unless explicit written permission is granted by the corresponding
   Copyright Holder. This restriction only applies to the primary font name as
   presented to the users.

4. The name(s) of the Copyright Holder(s) or the Author(s) of the Font
   Software shall not be used to promote, endorse or advertise any
   Modified Version, except to acknowledge the contribution(s) of the
   Copyright Holder(s) and the Author(s) or with their explicit written
   permission.

5. The Font Software, modified or unmodified, in part or in whole,
   must be distributed entirely under this license, and must not be
   distributed under any other license. The requirement for fonts to
   remain under this license does not apply to any document created using
   the fonts or their derivatives.

Termination
-----------

This license becomes null and void if any of the above conditions are
not met.

DISCLAIMER

THE FONT SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO ANY WARRANTIES OF
MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.
IN NO EVENT SHALL THE COPYRIGHT HOLDER BE LIABLE FOR ANY CLAIM, DAMAGES
OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE,
ARISING FROM, OUT OF OR IN CONNECTION WITH THE USE OR INABILITY TO USE
THE FONT SOFTWARE OR THE USE OF OTHER DEALINGS IN THE FONT SOFTWARE.
```
