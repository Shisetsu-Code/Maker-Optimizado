// Single initialization per browser realm. No dynamic code evaluation or network API.
import init, {normalize_content_json,normalize_design_json} from './wasm/maker_browser.js';
const wasm = new URL('./wasm/maker_browser_bg.wasm', import.meta.url);
if (typeof process !== 'undefined' && process.versions?.node) {
    const {readFile} = await import('node:fs/promises');
    await init({module_or_path:await readFile(wasm)});
} else {
    await init({module_or_path:wasm});
}
export const normalizeContentMeta = value => JSON.parse(normalize_content_json(JSON.stringify(value)));
export const normalizeDesignMeta = value => JSON.parse(normalize_design_json(JSON.stringify(value)));
