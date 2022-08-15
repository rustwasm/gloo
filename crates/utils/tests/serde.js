const wasm = require('wasm-bindgen-test.js');
const assert = require('assert');

exports.verify_serde = function(a) {
  assert.deepStrictEqual(a, {
    a: 0,
    b: 'foo',
    c: null,
    d: { a: 1 }
  });

  return {
    a: 2,
    b: 'bar',
    c: { a: 3 },
    d: { a: 4 },
  }
};
