# 和平精英 .imazingapp 配置生成器

一个配置驱动型的 `.imazingapp` 生成系统：用户通过微信小程序选择游戏配置，
小程序**本地**生成可通过 iMazing 导入 iOS 的配置包——**纯前端单机，无服务端**。

## 当前状态

纯前端单机方案已实现（服务端已移除）：配置与模板内嵌小程序包，
生成在设备本地完成，无网络请求、无部署、无运维。

完成流程：

    微信小程序
        ↓
    schema 驱动动态配置页面
        ↓
    engine/generate.js（本地生成）
        ↓
    translate → sav/ini 修改 → zip 字节重写
        ↓
    写本地缓存 → 分享到微信（iMazing 导入）

## 架构

### 小程序端（运行链路）

    miniprogram/
    ├── engine/
    │   ├── config.js         内嵌配置（由 scripts/build-config.js 生成）
    │   ├── template.js       预处理模板 base64 内嵌（由 scripts/build-template.js 生成）
    │   ├── translate.js      选择 → 补丁操作（对齐 Rust patch-engine）
    │   ├── ini.js            UserCustom.ini 行级处理（对齐 ini-engine）
    │   ├── sav.js            GVAS 定位 + 原地覆盖（不移植完整解析器）
    │   ├── zip.js            zip 字节级重写（零压缩代码）
    │   ├── filename.js       文件名生成
    │   ├── schema.js         内嵌配置 → 前端 schema 视图（对齐原服务端转换）
    │   └── generate.js       编排入口
    └── test/engine.test.js   Node 单测（对照 golden）

### Rust Workspace（开发期 oracle，不部署、不进包）

    devtools → pipeline → {patch-engine, save-engine, ini-engine, zip-engine, config-compiler}
                        → config-model → domain

-   domain：基础类型与错误定义
-   config-model：配置数据模型
-   config-compiler：配置加载、校验、编译
-   patch-engine：用户选择转换为 PatchOp
-   save-engine：UE .sav 文件修改（uesave）
-   ini-engine：ini 配置修改
-   zip-engine：zip 基础处理（骨架，pipeline 直接用 zip crate）
-   template-engine：imazingapp 模板流式生成
-   pipeline：生成流程编排
-   devtools：开发期工具（模板预处理 / 扫描断言 / golden 生成）

## 开发期工具（devtools）

```text
cargo run -p devtools -- preprocess <in.imazingapp> <out.imazingapp>
    模板预处理：目标条目（Active.sav / UserCustom.ini）改为 Stored 存储

node scripts/build-template.js <预处理后的模板>
    模板 base64 内嵌为小程序代码文件（engine/template.js），
    二进制文件在打包/真机传输不可靠，JS 内嵌保证必然随包

cargo run -p devtools -- scan <config_dir> <template.imazingapp>
    扫描断言：配置引用的 sav 属性必须存在且定长；缺失的 ini cvar
    会被报告，并在生成时自动插入（模板升级时第一时间暴露差异）

cargo run -p devtools -- golden <config_dir> <template.imazingapp> <out_dir>
    生成各配置组合的 golden fixtures（case-NN.imazingapp + case-NN.json），
    Node 单测以条目级（name/method/size/crc32）对照 Rust pipeline 产物
```

完整构建流程（CI 与本地一致）：

```text
cargo run -p devtools -- preprocess template.imazingapp .cache/template.imazingapp
node scripts/build-template.js .cache/template.imazingapp
node scripts/build-config.js
cargo run -p devtools -- scan config .cache/template.imazingapp
cargo run -p devtools -- golden config .cache/template.imazingapp golden
node miniprogram/engine/test/engine.test.js
```

## 配置系统

所有业务规则来自 `config/`：

    config/
    ├── schema.json
    ├── patches.json
    ├── presets.json
    ├── targets.json
    └── versions.json

支持配置（当前 10 项字段）：

-   画质
-   FPS
-   分辨率缩放
-   视觉风格
-   抗锯齿
-   阴影
-   渲染优化（开启写入 5 条 CVar，关闭移除）
-   亮度
-   镜头视野
-   冲刺灵敏度

新增配置原则：

1. 修改 `config/*.json`
2. 运行 `node scripts/build-config.js` 重新生成 `miniprogram/engine/config.js`
3. 运行 devtools scan / golden + Node 测试
4. 重新发布小程序

## 测试

-   Rust oracle 侧：`cargo test`（配置 / 引擎 / pipeline 测试）
-   前端引擎侧：见上方完整构建流程（golden + Node 对照）
-   GitHub Actions 门禁：fmt / clippy / test + 预处理 / 内嵌 / 扫描 / golden / Node 对照
-   真机验收：iMazing 导入 + 游戏内验证（发布前人工确认）

## 小程序

目录：

    miniprogram/

特点：

-   schema 自动生成配置 UI（select / switch / slider）
-   本地生成：无网络请求，生成即时完成
-   本地缓存最新一个文件，支持反复分享（无需重新生成）
-   激励视频广告解锁分享（完整观看后 24 小时免广告）
-   生成按钮 5 秒冷却防刷

## 开发要求

-   Rust 2024 Edition（oracle 侧）
-   配置驱动
-   禁止硬编码游戏规则
-   保持 Workspace 分层依赖
-   新功能优先扩展配置
