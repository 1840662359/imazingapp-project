// 小程序本地生成文件的缓存位置与读取方法。

const CACHE_FILE_PATH = `${wx.env.USER_DATA_PATH}/latest_config.imazingapp`;
const CACHE_NAME_KEY = 'latest_config_filename';

/** 读取已缓存的最新文件；没有缓存时返回 null。 */
function getCachedFile() {
  const fs = wx.getFileSystemManager();
  try {
    fs.accessSync(CACHE_FILE_PATH);
    const filename = wx.getStorageSync(CACHE_NAME_KEY) || 'template.imazingapp';
    return { filePath: CACHE_FILE_PATH, filename };
  } catch (e) {
    return null;
  }
}

module.exports = { CACHE_FILE_PATH, CACHE_NAME_KEY, getCachedFile };
