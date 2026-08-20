// miniprogram/engine/ini.js —— UserCustom.ini 行级处理。
//
// 对齐 Rust ini-engine 的 IniDoc：
// - load：跳过空行与 ;/# 注释，解析 [section] 与 +CVars=key=value
// - save：按 section 顺序输出，键排序，section 间空一行
// - 注释/空行在 load 时丢弃（与 Rust 行为一致，golden 对照保证）

'use strict';

const { utf8Decode, utf8Encode } = require('./util.js');

/** ini 文档（section 保序，entries 用 Map 保插入序、save 时排序）。 */
class IniDoc {
  constructor() {
    /** @type {Map<string, Map<string,string>>} */
    this.sections = new Map();
  }

  /** 从字节加载。 */
  static load(bytes) {
    let text = utf8Decode(bytes);
    // 剥离 UTF-8 BOM（部分编辑器保存时可能写入），否则首行 section 无法识别
    if (text.charCodeAt(0) === 0xfeff) {
      text = text.slice(1);
    }
    const doc = new IniDoc();
    let current = null;

    for (const rawLine of text.split('\n')) {
      const line = rawLine.trim();
      if (line === '' || line.startsWith(';') || line.startsWith('#')) continue;

      if (line.startsWith('[') && line.endsWith(']')) {
        current = line.slice(1, -1);
        if (!doc.sections.has(current)) {
          doc.sections.set(current, new Map());
        }
        continue;
      }

      if (line.startsWith('+CVars=')) {
        if (current === null) {
          throw new Error('Entry found before any section');
        }
        const entry = line.slice(7);
        const eq = entry.indexOf('=');
        if (eq === -1) {
          throw new Error(`Invalid entry format: ${line}`);
        }
        const key = entry.slice(0, eq);
        const value = entry.slice(eq + 1);
        doc.sections.get(current).set(key, value);
      }
    }
    return doc;
  }

  /** 设置值（section 不存在则创建）。 */
  set(section, key, value) {
    if (!this.sections.has(section)) {
      this.sections.set(section, new Map());
    }
    this.sections.get(section).set(key, value);
  }

  /** 删除键。 */
  remove(section, key) {
    const entries = this.sections.get(section);
    if (entries) entries.delete(key);
  }

  /** 读取值。 */
  get(section, key) {
    const entries = this.sections.get(section);
    return entries ? entries.get(key) : undefined;
  }

  /** 键是否存在。 */
  contains(section, key) {
    const entries = this.sections.get(section);
    return entries ? entries.has(key) : false;
  }

  /** 序列化为字节（与 Rust IniDoc::save 字节级一致）。 */
  save() {
    let out = '';
    for (const [section, entries] of this.sections) {
      out += `[${section}]\n`;
      const keys = Array.from(entries.keys()).sort();
      for (const key of keys) {
        out += `+CVars=${key}=${entries.get(key)}\n`;
      }
      out += '\n';
    }
    return utf8Encode(out);
  }
}

module.exports = { IniDoc };
