import { default as init } from '/pkg/webextension_demo.js';

init(browser.extension.getURL('/pkg/webextension_demo_bg.wasm'));
