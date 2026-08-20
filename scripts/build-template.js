// scripts/build-template.js —— 模板 base64 内嵌进 JS。
//
// 用法: node scripts/build-template.js <template.imazingapp>
// 输出: miniprogram/engine/template.js
//
// 原因：小程序打包/真机传输对未知扩展名的二进制文件（.imazingapp）
// 不可靠，base64 内嵌 JS 保证模板必然随代码打包、必然可读。

'use strict';

const fs = require('fs');
const path = require('path');

const input = process.argv[2];
if (!input) {
  console.error('用法: node scripts/build-template.js <template.imazingapp>');
  process.exit(1);
}
if (!fs.existsSync(input)) {
  console.error(`模板不存在: ${input}`);
  process.exit(1);
}

const buf = fs.readFileSync(input);
const outFile = path.join(__dirname, '..', 'miniprogram', 'engine', 'template.js');
const content = `// 由 scripts/build-template.js 生成，请勿手改。
// 模板字节（base64，${buf.length} 字节 → ${Math.ceil(buf.length / 3) * 4} 字符），
// 数据源变更后重新运行: node scripts/build-template.js <预处理后的模板>
'use strict';

module.exports = '${buf.toString('base64')}';
`;

fs.writeFileSync(outFile, content);
console.log(`✓ 已生成 ${path.relative(process.cwd(), outFile)}（${buf.length} 字节模板）`);
