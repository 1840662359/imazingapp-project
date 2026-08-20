# TODO · 项目进度

> 当前阶段：纯前端单机方案已实现，待真机验收与发布

## ✅ 已完成

### 纯前端单机迁移（核心架构变更）

-   [x] 方案确认：配置 + 模板全内嵌，零服务器（docs/pure-frontend-design.md）
    - 完成时间: 2026-08-20
    - 修改文件: docs/pure-frontend-design.md
-   [x] devtools crate：模板预处理（Stored 化）+ 扫描断言 + golden 生成
    - 完成时间: 2026-08-20
    - 修改文件: crates/devtools/*, Cargo.toml
-   [x] GVAS 布局实测校准：88 个属性定位全部验证（devtools dump --scan）
    - 完成时间: 2026-08-20
    - 修改文件: crates/devtools/src/dump.rs
-   [x] JS engine 模块：config/translate/ini/sav/zip/filename/generate
    - 完成时间: 2026-08-20
    - 修改文件: miniprogram/engine/*
-   [x] Node 单测对照 golden：18/18 通过（条目级 name/method/size/crc32）
    - 完成时间: 2026-08-20
    - 修改文件: miniprogram/engine/test/engine.test.js
-   [x] 页面改造：index.js 改调本地生成（无网络请求）
    - 完成时间: 2026-08-20
    - 修改文件: miniprogram/pages/index/index.js
-   [x] 服务端清理：删除 crates/server、Dockerfile、translations 死数据
    - 完成时间: 2026-08-20
    - 修改文件: crates/server/*, Dockerfile, config/translations/*
-   [x] CI 更新：去掉 Docker 校验，加入模板预处理 + golden + Node 对照
    - 完成时间: 2026-08-20
    - 修改文件: .github/workflows/ci.yml

### Rust 后端（原线上版本，保留为开发期 oracle）

-   [x] Rust Workspace 架构完成
-   [x] 10 个 crate 模块拆分完成（server 已随迁移移除）
-   [x] 配置中心完成
-   [x] schema/patch/target/translation 加载完成
-   [x] Patch Engine 完成
-   [x] Save Engine 完成
-   [x] Ini Engine 完成
-   [x] Template Engine 完成
-   [x] Pipeline 全流程串联完成
-   [x] Axum API 完成
-   [x] Docker 构建支持完成
-   [x] 移除存储层，`/build` 直接返回文件字节（无状态、可多实例）
-   [x] 移除 IP 限流（callContainer 内网直达，来源为网关 IP）
-   [x] 微信云托管部署完成
-   [x] GitHub Actions CI（fmt / clippy / test + Docker 构建校验）
-   [x] 并发加固：build 走 spawn_blocking + 信号量闸门 + 饱和 503
-   [x] 模板免重压缩（raw_copy_file / Stored），降低单请求 CPU
-   [x] 清理死依赖（uuid、server 未用的 zip-engine）

### 配置能力

-   [x] FPS 配置
-   [x] 画质配置
-   [x] 分辨率缩放
-   [x] 亮度调整
-   [x] 阴影设置
-   [x] 抗锯齿设置
-   [x] 视觉风格
-   [x] 镜头视野
-   [x] 冲刺灵敏度
-   [x] 渲染优化（开启写入 5 条 CVar，关闭移除 · remove_when）

### 微信小程序

-   [x] schema 动态渲染
-   [x] 配置选择页面
-   [x] 下载/分享页面
-   [x] 文件分享流程
-   [x] 后端 API 对接（改用 wx.cloud.callContainer，已随迁移废弃）
-   [x] 本地缓存最新文件，支持反复分享
-   [x] 生成按钮 5 秒冷却防刷
-   [x] 激励视频广告解锁分享（完整观看后 24 小时免广告）

### 真机验证与上线（原线上版本）

-   [x] 完整测试生成流程
-   [x] iMazing 导入测试
-   [x] 游戏内验证配置生效
-   [x] 微信审核发布上线

------------------------------------------------------------------------

## 🟢 后续任务

### 纯前端方案收尾（当前优先级）

-   [x] 真机验收：本地生成 + 分享链路验证通过
    - 完成时间: 2026-08-20
    - 修复: schema 本地加载、模板 base64 内嵌、writeFileSync ArrayBuffer 转换
-   [ ] 微信审核发布（纯前端版本）
-   [ ] `zip-engine` crate 无人依赖：删除或保留待定（删除属架构调整，需同步 CLAUDE.md）

### Save Engine（oracle 侧）

-   [ ] 支持枚举属性写入
-   [ ] 支持新增属性
-   [ ] 增强 UE Save 格式兼容

### 商业化

-   [ ] 下载统计（本地事件上报，如可接受静态托管）
-   [ ] 用户体系

------------------------------------------------------------------------

## 当前架构原则

不要通过增加 Rust 判断支持新配置。

正确方式：

    新增功能
       ↓
    修改 config/
       ↓
    node scripts/build-config.js
       ↓
    devtools scan / golden + Node 测试
       ↓
    复用 Engine（小程序本地生成）

只有当出现新的执行能力时，才新增 Engine。
