use gloo_worker::Codec;

pub struct TransferrableCodec {}

// This codec implementation relys on some internal implementation details about gloo worker message types.
// This should be considered as a last resort approach after all other options are exhausted.
//
// In addition, using this approach could also potentially create unsound code.
// This could cause data being sent to a wrong worker or with wrong handler id if it is not implemented
// properly.
//
// You should only use this approach if you know how to make it sound.
// This example mitigates this issue by only allowing 1 file to be processed at a time
// and only 1 worker instance across the entire tab.
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
