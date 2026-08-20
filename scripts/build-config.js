// scripts/build-config.js —— 从 config/*.json 生成小程序内嵌配置。
//
// 用法: node scripts/build-config.js
// 输出: miniprogram/engine/config.js（配置变更后重跑即可）
//
// 纯前端单机方案：配置随包发布，改动配置需重新生成 + 发版。

const fs = require('fs');
const path = require('path');

const root = path.join(__dirname, '..');
const configDir = path.join(root, 'config');
const outFile = path.join(root, 'miniprogram', 'engine', 'config.js');

const files = ['schema.json', 'patches.json', 'targets.json', 'versions.json'];
const data = {};
for (const f of files) {
  data[f.replace('.json', '')] = JSON.parse(fs.readFileSync(path.join(configDir, f), 'utf8'));
}

const content = `// 由 scripts/build-config.js 从 config/*.json 自动生成，请勿手改。
// 数据源变更后重新运行: node scripts/build-config.js
'use strict';

module.exports = ${JSON.stringify(data, null, 2)};
`;

fs.writeFileSync(outFile, content);
console.log(`✓ 已生成 ${path.relative(root, outFile)}`);
