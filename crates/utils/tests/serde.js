function deepStrictEqual(left, right) {
  var left_json = JSON.stringify(left);
  var right_json = JSON.stringify(right);
  if (left_json !== right_json) {
    throw Error(`${left_json} != ${right_json}`)
  }
}

export function verify_serde (a) {
  deepStrictEqual(a, {
    a: 0,
    b: 'foo',
    c: null,
    d: { a: 1 }
  });
};

export function make_js_value() {
  return {
    a: 2,
    b: 'bar',
    c: { a: 3 },
    d: { a: 4 },
  }
};
