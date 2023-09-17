use gloo_worker::Codec;

pub struct TransferrableCodec {}

// This codec implementation relys on some internal implementation details about gloo worker message types.
// Fields marked with `#[serde(with = "serde_wasm_bindgen::preserve")]` will be passed as-is.
impl Codec for TransferrableCodec {
    fn encode<I>(input: I) -> wasm_bindgen::JsValue
    where
        I: serde::Serialize,
    {
        serde_wasm_bindgen::to_value(&input).expect("failed to encode")
    }

    fn decode<O>(input: wasm_bindgen::JsValue) -> O
    where
        O: for<'de> serde::Deserialize<'de>,
    {
        serde_wasm_bindgen::from_value(input).expect("failed to decode")
    }
}
