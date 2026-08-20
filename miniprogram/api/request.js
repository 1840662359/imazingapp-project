// api/request.js - 网络请求封装（微信云托管 callContainer）

const app = getApp();

/**
 * 统一调用云托管容器
 * @param {string} path  容器内路径，如 '/api/v1/schema'
 * @param {object} opts  { method, data, responseType }
 * 返回整个 res（含 data / header / statusCode）
 */
function callContainer(path, opts = {}) {
  return new Promise((resolve, reject) => {
    wx.cloud.callContainer({
      config: { env: app.globalData.cloudEnv },
      path,
      method: opts.method || 'GET',
      data: opts.data,
      responseType: opts.responseType, // 二进制下载时传 'arraybuffer'
      header: {
        'X-WX-SERVICE': app.globalData.serviceName, // 必填：指定服务名
        'content-type': 'application/json'
      },
      success(res) {
        if (res.statusCode >= 200 && res.statusCode < 300) {
          resolve(res);
        } else {
          const msg = (res.data && res.data.message) || `请求失败 ${res.statusCode}`;
          reject(new Error(msg));
        }
      },
      fail(err) {
        reject(err);
      }
    });
  });
}

/**
 * GET 请求，返回 res.data
 */
function get(path) {
  return callContainer(path, { method: 'GET' }).then(res => res.data);
}

/**
 * POST 请求，返回 res.data
 */
function post(path, data) {
  return callContainer(path, { method: 'POST', data }).then(res => res.data);
}

// 固定的本地缓存路径：只保留最新一个文件，每次生成覆盖。
// 用纯 ASCII 名（中文/% 会导致 shareFileMessage 出错），中文显示名单独存。
// 由页面层（index.js 写入 / download.js 经 getCachedFile 读取）共用，勿改。
const CACHE_FILE_PATH = `${wx.env.USER_DATA_PATH}/latest_config.imazingapp`;
const CACHE_NAME_KEY = 'latest_config_filename';

/**
 * 构建并缓存文件：POST 生成，直接拿 ArrayBuffer 写入固定缓存路径。
 * @param {object} data  { selections } 或 { preset_id }
 * @returns {Promise<{ filePath, filename }>}
 */
function buildFile(path, data) {
  return callContainer(path, {
    method: 'POST',
    data,
    responseType: 'arraybuffer'
  }).then(res => {
    // 1. 从响应头解析中文文件名
    let filename = 'template.imazingapp';
    const cd = res.header['content-disposition'] || res.header['Content-Disposition'];
    if (cd) {
      const m = cd.match(/filename="?([^";\n]+)"?/);
      if (m && m[1]) {
        filename = m[1];
        try {
          filename = decodeURIComponent(m[1]);
        } catch (e) {
          filename = m[1];
        }
      }
    }

    // 2. 写入固定缓存路径（覆盖上一个），并记住显示名
    const fs = wx.getFileSystemManager();
    fs.writeFileSync(CACHE_FILE_PATH, res.data); // res.data 是 ArrayBuffer
    wx.setStorageSync(CACHE_NAME_KEY, filename);

    return { filePath: CACHE_FILE_PATH, filename };
  });
}

/**
 * 读取已缓存的最新文件（若存在）。
 * @returns {{ filePath, filename } | null}
 */
function getCachedFile() {
  const fs = wx.getFileSystemManager();
  try {
    fs.accessSync(CACHE_FILE_PATH); // 文件存在
    const filename = wx.getStorageSync(CACHE_NAME_KEY) || 'template.imazingapp';
    return { filePath: CACHE_FILE_PATH, filename };
  } catch (e) {
    return null; // 无缓存
  }
}

module.exports = {
  get,
  post,
  buildFile,
  getCachedFile,
  CACHE_FILE_PATH,
  CACHE_NAME_KEY
};
