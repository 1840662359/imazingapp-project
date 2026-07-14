// api/config.js - 配置相关 API

const request = require('./request.js');

/**
 * 获取配置 schema
 */
function getSchema() {
  return request.get('/api/v1/schema');
}

/**
 * 获取预设列表
 */
function getPresets() {
  return request.get('/api/v1/presets');
}

/**
 * 构建配置文件：后端直接返回文件字节，写入本地缓存（只留最新一个）
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
