# 微信小程序（纯前端单机版）

> 本版本无服务端：配置与模板内嵌于包内，生成在设备本地完成。
> 生成、配置读取和文件缓存均在本地完成，不包含云托管请求。

## 前置要求

1. 安装[微信开发者工具](https://developers.weixin.qq.com/miniprogram/dev/devtools/download.html)
2. 注册微信小程序账号（或使用测试号）

## 快速启动

1. 打开微信开发者工具
2. 点击"导入项目"，选择目录：`d:\imazingapp-project\miniprogram`
3. 填写 AppID（有账号填实际 AppID，测试选"测试号"）
4. 点击"编译"

无需启动任何服务、无需配置域名。

## 目录结构

```
miniprogram/
├── engine/                      # 纯前端生成引擎（核心）
│   ├── config.js                # 内嵌配置（scripts/build-config.js 生成）
│   ├── template.js              # 预处理模板 base64 内嵌（scripts/build-template.js 生成）
│   ├── translate.js             # 选择 → 补丁操作
│   ├── ini.js                   # UserCustom.ini 行级处理
│   ├── sav.js                   # GVAS 定位 + 原地覆盖
│   ├── zip.js                   # zip 字节级重写
│   ├── filename.js              # 文件名生成
│   ├── schema.js                # 内嵌配置 → 前端 schema 视图
│   ├── util.js                  # UTF-8 / base64 / TypedValue 工具
│   ├── generate.js              # 编排入口
│   └── test/engine.test.js      # Node 单测（对照 golden，见仓库根 README）
│
├── utils/
│   └── file-cache.js            # 本地生成文件缓存路径与读取方法
│
├── components/
│   └── dynamic-field/           # schema 动态字段组件
│
├── pages/
│   ├── index/                   # 首页（配置选择 + 本地生成）
│   └── download/                # 分享页（激励视频解锁 + shareFileMessage）
│
├── app.js                       # 应用入口（无全局初始化）
├── app.json
├── app.wxss
└── project.config.json
```

## 使用流程

1. **首页**：schema 本地加载，渲染动态字段（select / switch / slider）
2. **选择配置**：调整滑块、切换开关
3. **生成**：点击"生成配置文件"——本地执行生成（无网络请求），
   产物写入本地缓存（`${USER_DATA_PATH}/latest_config.imazingapp`）
4. **分享**：跳转分享页——首次需完整观看激励视频，解锁后 24 小时免广告，
   `wx.shareFileMessage` 分享文件，接收方用 iMazing 导入

## 常见问题

### Q: 首页提示"模板数据缺失"

**A:** `engine/template.js` 缺失或为空。重新运行构建流程：
`node scripts/build-template.js <预处理模板>`（见仓库根 README），
然后重新编译上传。

### Q: 生成提示"生成失败"

**A:** 打开调试器（Console）查看具体错误。常见原因：
- 配置与模板版本不匹配（改过 config 后未重新生成 engine/config.js）
- 模板未预处理（需先运行 devtools preprocess）

### Q: 分享按钮无反应

**A:** 检查激励视频广告单元 ID 是否有效（`download.js` 中的 `adUnitId`），
广告加载失败时会自动放行分享。

## 调试

- **Console**：查看日志和错误
- **Storage**：查看本地缓存（`latest_config_filename` 等）
- **AppData**：查看页面数据
- 真机调试：点击"真机调试"→ 扫码 → 查看真机 Console

## 部署上线

1. 开发者工具右上角"上传"，填写版本号
2. 小程序后台：**版本管理** → **开发版本** → **提交审核**
3. 审核通过后发布

## 扩展开发

新增配置能力：修改仓库根 `config/*.json` → 运行
`node scripts/build-config.js` → 重新生成内嵌配置 → 重新编译上传。
引擎行为变更须先改 Rust oracle 并重新生成 golden（见仓库根 README）。
