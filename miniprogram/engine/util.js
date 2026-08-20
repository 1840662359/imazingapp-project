// miniprogram/engine/util.js —— 基础工具（UTF-8 编解码、TypedValue 工具）。
// 纯函数模块，不依赖 wx API，可被 Node 直接测试。

'use strict';

/**
 * Uint8Array → UTF-8 字符串（小程序基础库无 TextDecoder 的兜底实现）。
 * 支持多字节 UTF-8（ini 文件实际为 ASCII，此处覆盖一般情况）。
 */
function utf8Decode(bytes) {
  let out = '';
  let i = 0;
  while (i < bytes.length) {
    const b = bytes[i];
    if (b < 0x80) {
      out += String.fromCharCode(b);
      i += 1;
    } else if (b < 0xe0) {
      const c = ((b & 0x1f) << 6) | (bytes[i + 1] & 0x3f);
      out += String.fromCharCode(c);
      i += 2;
    } else if (b < 0xf0) {
      const c = ((b & 0x0f) << 12) | ((bytes[i + 1] & 0x3f) << 6) | (bytes[i + 2] & 0x3f);
      out += String.fromCharCode(c);
      i += 3;
    } else {
      // 4 字节（代理对）——本项目不会出现，简单降级
      const c = ((b & 0x07) << 18) | ((bytes[i + 1] & 0x3f) << 12) |
        ((bytes[i + 2] & 0x3f) << 6) | (bytes[i + 3] & 0x3f);
      out += String.fromCharCode(c);
      i += 4;
    }
  }
  return out;
}

/**
 * UTF-8 字符串 → Uint8Array。
 */
function utf8Encode(str) {
  const out = [];
  for (let i = 0; i < str.length; i++) {
    let c = str.charCodeAt(i);
    if (c < 0x80) {
      out.push(c);
    } else if (c < 0x800) {
      out.push(0xc0 | (c >> 6), 0x80 | (c & 0x3f));
    } else {
      out.push(0xe0 | (c >> 12), 0x80 | ((c >> 6) & 0x3f), 0x80 | (c & 0x3f));
    }
  }
  return new Uint8Array(out);
}

/**
 * base64 → Uint8Array。
 * 小程序基础库不保证 atob 可用，手写解码（模板以 base64 内嵌于 JS，
 * 避免二进制文件在打包/真机传输时丢失）。
 */
const B64_CHARS = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/';
function base64ToBytes(str) {
  str = str.replace(/\s/g, '');
  let pad = 0;
  if (str.charCodeAt(str.length - 1) === 61 /* '=' */) pad++;
  if (str.charCodeAt(str.length - 2) === 61) pad++;
  const out = new Uint8Array(((str.length * 3) >> 2) - pad);
  let buffer = 0;
  let bits = 0;
  let o = 0;
  for (let i = 0; i < str.length; i++) {
    const idx = B64_CHARS.indexOf(str[i]);
    if (idx === -1) continue;
    buffer = (buffer << 6) | idx;
    bits += 6;
    if (bits >= 8) {
      bits -= 8;
      out[o++] = (buffer >> bits) & 0xff;
    }
  }
  return out;
}

/**
 * TypedValue 表示：{ t: 'int'|'float'|'bool'|'str'|'enum', v: 值 }。
 * 与 Rust 的 domain::TypedValue 一一对应（ini/sav 引擎的通用契约）。
 */

/** 按 value_type 将 JS 原生值（JSON 解析产物）转为 TypedValue。 */
function toTypedValue(raw, valueType) {
  switch (valueType) {
    case 'int':
      if (typeof raw === 'number' && Number.isFinite(raw)) {
        return { t: 'int', v: Math.trunc(raw) };
      }
      throw new Error(`expected int, got ${JSON.stringify(raw)}`);
    case 'float':
      if (typeof raw === 'number' && Number.isFinite(raw)) {
        return { t: 'float', v: raw };
      }
      throw new Error(`expected float, got ${JSON.stringify(raw)}`);
    case 'bool':
      if (typeof raw === 'boolean') {
        return { t: 'bool', v: raw };
      }
      throw new Error(`expected bool, got ${JSON.stringify(raw)}`);
    case 'str':
      if (typeof raw === 'string') {
        return { t: 'str', v: raw };
      }
      throw new Error(`expected string, got ${JSON.stringify(raw)}`);
    case 'enum':
      throw new Error('enum type cannot use raw value');
    default:
      throw new Error(`unknown value_type: ${valueType}`);
  }
}

/** 将字符串形式的用户输入按类型解析为 TypedValue（对齐 Rust parse_value_from_string）。 */
function parseValueFromString(input, valueType) {
  switch (valueType) {
    case 'int': {
      const v = Number(input);
      if (!Number.isInteger(v)) throw new Error(`cannot parse '${input}' as int`);
      return { t: 'int', v };
    }
    case 'float': {
      const v = Number(input);
      if (!Number.isFinite(v)) throw new Error(`cannot parse '${input}' as float`);
      return { t: 'float', v };
    }
    case 'bool': {
      if (input === 'true') return { t: 'bool', v: true };
      if (input === 'false') return { t: 'bool', v: false };
      throw new Error(`cannot parse '${input}' as bool`);
    }
    case 'str':
      return { t: 'str', v: input };
    case 'enum':
      throw new Error('enum type cannot use value_from_input');
    default:
      throw new Error(`unknown value_type: ${valueType}`);
  }
}

/** TypedValue → ini 字符串（对齐 Rust pipeline::typed_value_to_string）。 */
function typedValueToString(tv) {
  switch (tv.t) {
    case 'str': return tv.v;
    case 'int': return String(tv.v);
    case 'float': return String(tv.v); // JS Number.toString 与 Rust f64 to_string 对 IEEE754 一致
    case 'bool': return tv.v ? '1' : '0';
    case 'enum': return String(tv.v);
    default: throw new Error(`unknown TypedValue type: ${tv.t}`);
  }
}

module.exports = { utf8Decode, utf8Encode, base64ToBytes, toTypedValue, parseValueFromString, typedValueToString };
