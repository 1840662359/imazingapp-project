# 纯前端单机方案设计：小程序本地生成 `.imazingapp`

> 状态：已确认（2026-08-20）
> 决策 1：配置 + 模板全内嵌，零服务器（方案 A）
> 决策 2：Rust 仓库保留为开发期 oracle（golden 基准 + 模板预处理工具），不部署、不进包

## 1. 背景与目标

现状：生成链路依赖 Rust 服务端（微信云托管）。目标：移除服务端，改为小程序本地生成。

- **目标**：移除云托管、Docker、扩容、503 逻辑等一切服务端设施，纯前端单机生成。
- **不变**：schema 动态渲染、5 秒冷却、激励视频解锁、本地缓存、`wx.shareFileMessage` 分享。
- **保留**：`config/` 配置驱动原则、模块隔离原则（只是引擎从 Rust 换成 JS）。

## 2. 可行性数据（实测）

| 实测项 | 数据 | 意义 |
|---|---|---|
| template.imazingapp | 268KB / 55 条目 | 远小于包体限额，可内嵌 |
| Active.sav | 5,481 字节 | JS 处理无压力 |
| UserCustom.ini | 10,055 字节 | 纯文本 |
| 5 个 sav 目标属性 | 全部以 ASCII FString 存在于字节流 | 定位 + 原地覆盖可行 |
| sav 属性结构 | `len+名称+NUL / len+类型+NUL / size / 值`，Int/Float 定长 4B | 覆盖不改结构 |
| 模板压缩方式 | sav/ini 为 Deflate | 靠模板预处理绕开压缩算法 |

## 3. 架构总览

```
用户选择 (selections)
    ↓
translate.js（纯配置查表，替代 patch-engine）
    ↓
ops[]（Set / Remove）
    ↓
按 target 分组 → ini.js / sav.js
    ↓
zip.js 字节重写（替代 template-engine）
    ↓
ArrayBuffer → 写缓存路径 → 分享（现有链路不动）
```

| 现有 Rust crate | 前端替代 |
|---|---|
| config-compiler + patch-engine | translate.js + 内嵌 config JSON（编译期校验→开发期脚本） |
| save-engine (uesave) | sav.js 定位覆盖（不移植 uesave） |
| ini-engine | ini.js 行级处理 |
| template-engine | zip.js 字节重写（零压缩代码） |
| pipeline | generate.js 同步函数链 |
| server / routes | 删除（文件名生成移入 filename.js） |

## 4. 模板预处理（构建期工具，Rust 开发期承担）

把模板内 `Active.sav`、`UserCustom.ini` 两个条目改为 **Stored（不压缩）** 后随包发布。

- 目的：JS 侧读两个目标文件无需 inflate；zip 重写全程仅字节搬运 + CRC32。
- 代价：包体增加约 11.5KB，可忽略。
- 输出产物语义与当前一致（Rust pipeline 本就以 Stored 写回这两个条目）。
- 同时生成模板扫描报告：断言 5 个 sav 属性存在且值定长；报告缺失的 ini cvar，缺失项由生成引擎自动插入——模板升级时第一时间暴露差异。

## 5. 核心模块设计

### 5.1 sav.js：定位 + 原地覆盖

扫描属性序列：匹配名称 FString → 读类型 → 读 size → 得值偏移 → 按类型覆盖定长值（Int/Float = 4B LE）。

- 只覆盖、不增删：不触碰对齐、不重排、不新增属性。
- 护栏：写入前校验 size 与期望类型一致，不一致返回「模板版本不兼容」，绝不盲写。
- 不支持：完整 GVAS 解析、sav Remove（当前配置无此绑定）。

### 5.2 zip.js：字节级重写

解析 EOCD → 中央目录 → 逐条目：

- 未改动条目：复制 local header + 原始压缩字节，修正偏移。
- 两个目标条目：读明文（Stored）→ 修改 → Stored 写回，重算 CRC32（表格法）。
- 重建中央目录 + EOCD。纯 DataView/Uint8Array，约 300 行。

### 5.3 ini.js

解析全部 section、保序、键排序后重写——行为与 Rust `IniDoc` 对齐（注释/空行在 load 时丢弃，save 时 section 间空一行）。

### 5.4 translate.js + filename.js

- translate：`(config, selections) → ops[]`，Set/Remove 两态，含 `value_from_input` 与 `value_map` 查表。
- filename：移植 `generate_filename`；分辨率最多保留两位小数并去除末尾零。

### 5.5 配置内嵌与自检

schema / patches / targets 编译进包，页面加载期轻量自检（字段/选项引用一致），失败提示「配置版本异常」。

## 6. 更新策略（已确认：方案 A 全内嵌）

- 配置、模板随小程序发版更新；新配置项/新模板需重新提审。
- 资源获取层留好接口，未来如需免审更新再补静态托管路径（B 方案，本设计不做）。

## 7. 测试与回归策略

1. **Rust 开发期 oracle**：保留 10 个 crate，`cargo test` 生成各配置组合 golden fixture。
2. **JS 单测对照**：逐条目对比 name / size / CRC32 / zip 有效性（字节级精确对比不可行——Rust 输出时间戳不固定，条目级等价已足够强）。
3. **真机验收**：iMazing 导入 + 游戏内验证（沿用原上线流程）。
4. 改模板/配置 → 重生成 golden → 跑 JS 测试，纳入开发脚本。

## 8. 风险与对策

| 风险 | 对策 |
|---|---|
| GVAS 变体（UTF-16 属性名等） | 当前模板全 ASCII；扫描脚本断言，模板升级自动暴露 |
| sav 属性缺失/改名 | 扫描脚本 + 运行时 size 护栏 |
| 新游戏版本 → 新模板 | 重新预处理 + 发版（方案 A 固有代价） |
| 与服务器产物行为不一致 | 条目级 golden 对照 + 真机验收 |
| 包体未来膨胀 | 268KB 远低于限额；预留分包方案（模板超 1MB 时拆分包） |

## 9. 实施阶段（未来按此推进）

- **阶段 0 开发期工具**：模板预处理脚本（Stored 化 + 扫描报告）、golden fixture 生成。
- **阶段 1 engine 模块**：`miniprogram/engine/` 下 zip.js / sav.js / ini.js / translate.js / filename.js + 单测对照 golden。
- **阶段 2 页面改造**：`index.js` 改调本地 `generate.js`；`request.js` 仅保留（B 方案预留，当前无用）；下载页不动。
- **阶段 3 清理**：删除 crates/server、Dockerfile、CI 服务端部分；移除 translations 死数据；README/TODO 同步（pipeline 保留——golden 生成依赖它）。
- **阶段 4 真机验收**：iMazing 导入 + 游戏内验证。

## 10. 交付物清单

- 新增 `miniprogram/engine/`：zip.js、sav.js、ini.js、translate.js、filename.js、config（内嵌 JSON）
- 新增构建期脚本：模板预处理、golden 生成（Rust 侧）
- 删除：crates/server、Dockerfile、CI 服务端部分、translations 死数据；保留全部引擎 crate 与 pipeline（oracle/golden 依赖）
