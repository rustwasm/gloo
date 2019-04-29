## Summary

Safe, rusty wrapper for the web-sys WebGL functionality.

What is included?
* Only standard rust types in the interface.
* Type safe interface.
* Error handling by returning results from all functions that can cause a runtime error.
* Simplification of the interface wherever it does not limit the functionality. (note that this opinion differ a bit from my previous post, I think it is not possible to have a 1-1 correspondence to the WebGL API, but it should be as close as possible)

What is not included?
* WebGL is a state machine which is not really aligned with the rust way of thinking. Unfortunately, there is not a straight forward and unbiased way to handle that. Therefore, I suggest to leave that to the higher level crates.
* Performance is ensured only when it is really easy to do, nothing more.

## Motivation

The web-sys WebGL functionality is difficult to approach because of the JS types, conversions and memory management. Also, the WebGL API in itself is very far from the Rust way of thinking, for example is it not type safe at all. So the purpose of this crate is to expose a simple and safe API which is similar, though not identical, to the WebGL API. The intended users are graphics programmers at all levels of experience.

## Detailed explanation

Here’s an example implementation: 
```rust

use web_sys;

// There should be a struct for WebGL 2 context too.
pub struct WebGlRenderingContext
{
    context: web_sys::WebGlRenderingContext
}

impl WebGlRenderingContext {
    // This is the only place where the API is web-sys specific and it should preferably be
    // used primarily from the Gloo canvas crate or something, not directly from the user.
    pub fn new(web_sys_context: web_sys::WebGlRenderingContext) -> Self
    {
        WebGlRenderingContext { context: web_sys_context }
    }
}

// Then it is 'just' implementing all of the webgl methods from web-sys. Let's start with a
// simple example.

impl WebGlRenderingContext {
    // Changing the input types to make sure that the method does not throw a runtime exception
    // is an easy win. No other errors can occur in this method so no need to return a result.
    pub fn viewport(&self, x: i32, y: i32, width: u32 /*changed from i32*/, height: u32/*changed from i32*/)
    {
        // Maybe check if the viewport has changed?
        self.context.viewport(x, y, width as i32, height as i32);
    }
}

// And another example:

#[derive(Debug, Eq, PartialEq)]
pub enum BlendType
{
    Zero,
    One,
    SrcColor,
    OneMinusSrcColor,
    SrcAlpha,
    OneMinusSrcAlpha,
    DstAlpha,
    OneMinusDstAlpha,
    ConstantColor,
    ConstantAlpha
    // ...
}

impl WebGlRenderingContext {
    // Again, the input parameter types makes the interface safe
    pub fn blend_func(&self, s_factor: BlendType, d_factor: BlendType) -> Result<(), Error>
    {
        // Check if ConstantColor and ConstantAlpha is used together (see https://developer.mozilla.org/en-US/docs/Web/API/WebGLRenderingContext/blendFunc)
        if s_factor == BlendType::ConstantColor && d_factor == BlendType::ConstantAlpha ||
            s_factor == BlendType::ConstantAlpha && d_factor == BlendType::ConstantColor
        {
            return Err(Error::WebGLError { message: "blend_func cannot be called with both ConstantColor and ConstantAlpha.".to_string() });
        }
        // Check if the blend state is already the desired blend state, if it is, then we don't call the webgl function!
        // ...
        Ok(())
    }
}

// So next example is a bit more complex and this involves exposing another API than the one in web-sys/WebGL.
// The goal here is not to copy the WebGL API, but rather to expose the exact same functionality as safely as possible.

pub struct VertexShader<'a>
{
    shader: web_sys::WebGlShader,
    context: &'a web_sys::WebGlRenderingContext
}

// Delete the shader after it has been used and possibly reused.
impl<'a> std::ops::Drop for VertexShader<'a>
{
    fn drop(&mut self)
    {
        self.context.delete_shader(Some(&self.shader));
    }
}

impl WebGlRenderingContext {
    pub fn create_vertex_shader(&self, source: &str) -> Result<VertexShader, Error>
    {
        let shader = self.context.create_shader(web_sys::WebGlRenderingContext::VERTEX_SHADER)
            .ok_or(Error::WebGLError {message: "Cannot create vertex shader!".to_string()})?;
        self.context.shader_source(&shader, source);
        self.context.compile_shader(&shader);

        if self.context.get_shader_parameter(&shader, web_sys::WebGlRenderingContext::COMPILE_STATUS).as_bool().unwrap_or(false)
        {
            Ok(VertexShader {shader, context: &self.context})
        } else {
            Err(Error::WebGLError { message: self.context.get_shader_info_log(&shader).unwrap_or_else(|| "Unknown error creating shader".into()) })
        }
    }

    pub fn create_program(&self, vertex_shader: VertexShader /*, fragment_shader: FragmentShader*/)
    {
        // ...
    }
}

#[derive(Debug)]
pub enum Error {
    WebGLError {message: String}
}

```


## Drawbacks, rationale and alternatives

This crate solves the easy problems with the web-sys (and WebGL) interface (rusty safe interface). The more hard problems (simple to use, avoid state machine, performance etc.) is difficult to address in general and especially without creating an opinionated library. Therefore, I envision this crate to be the foundation for a multitude of different opinionated libraries as well as to be used by graphics programmers that want low level control.

The alternative, as I see it, is to go straight to the higher level crates and then let graphics programmers use the web-sys API. However, I would have appreciated a crate like this a few months ago and I am probably not the only one.

## Unresolved questions

* IMO, the wrapper should be as safe and simple as possible, but not limit the programmer in any way. This means that you should be able to do exactly the same as with the web-sys API, and that might make the API a bit uglier, but how bad it is going to be, I don’t know yet. In the cases where it is inevitably going to be an ugly API, I want it to be as similar to the WebGL API as possible.
* Probably a lot more that I can’t think of right now…