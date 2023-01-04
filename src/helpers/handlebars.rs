use std::boxed::Box;

use handlebars::{
    Context, Decorator, Handlebars, Helper, HelperResult, Output, RenderContext, RenderError,
};
use serde_json::value::Value as Json;
use url::form_urlencoded;

// a decorator mutates current context data
fn set_decorator(
    d: &Decorator,
    _: &Handlebars,
    ctx: &Context,
    rc: &mut RenderContext,
) -> Result<(), RenderError> {
    // get the input of decorator
    let data_to_set = d.hash();
    // retrieve the json value in current context
    let ctx_data = ctx.data();

    if let Json::Object(m) = ctx_data {
        let mut new_ctx_data = m.clone();

        for (k, v) in data_to_set {
            new_ctx_data.insert(k.to_string(), v.value().clone());
        }

        rc.set_context(Context::wraps(new_ctx_data)?);
        Ok(())
    } else {
        Err(RenderError::new("Cannot extend non-object data"))
    }
}

// implement via bare function
fn encode(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let param = h.param(0).and_then(|v| v.value().as_str()).unwrap_or("");
    let encoded: String = form_urlencoded::Serializer::new(String::new())
        .append_pair("x", param)
        .finish();
    out.write(encoded.get(2..).unwrap_or(""))?;
    Ok(())
}

pub fn compile_template(
    template: &str,
    context: &serde_yaml::Value,
) -> Result<String, RenderError> {
    let mut hbar = Handlebars::new();
    hbar.register_decorator("set", Box::new(set_decorator));
    hbar.register_helper("encode", Box::new(encode));
    hbar.render_template(template, &context)
}
