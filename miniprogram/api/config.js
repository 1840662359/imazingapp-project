// 历史云托管 API 封装：schema、预设和构建方法已不在当前页面流程中使用；
// 下载页仍通过 getCachedFile() 读取本地生成的文件。

const request = require('./request.js');

/**
 * 从旧服务端获取配置 schema（当前页面使用本地内嵌配置）。
 */
function getSchema() {
  return request.get('/api/v1/schema');
}

/**
 * 从旧服务端获取预设列表（当前 schema 视图不提供服务端预设）。
 */
function getPresets() {
  return request.get('/api/v1/presets');
}

/**
 * 旧服务端构建流程：后端返回文件字节并写入本地缓存；当前生成由 engine/generate.js 在本地完成。
 * @param {Object} params - { preset_id?: string, selections?: Object }
 * @returns {Promise<{ filePath, filename }>}
 */
function build(params) {
  return request.buildFile('/api/v1/build', params);
}

/**
 * 读取已缓存的最新文件（若存在）
 * @returns {{ filePath, filename } | null}
 */
function getCachedFile() {
  return request.getCachedFile();
}

module.exports = {
  getSchema,
  getPresets,
  build,
  getCachedFile
};
