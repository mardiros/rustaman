use handlebars::{
    Context, BlockContext, Handlebars, Helper, HelperResult, Output, RenderContext, TemplateRenderError,
};
use std::boxed::Box;
use url::form_urlencoded;

// implement via bare function
fn set_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    rc: &mut RenderContext,
    _out: &mut dyn Output,
) -> HelperResult {
    let mut block_context = BlockContext::new();
    for (key, val) in h.hash().iter() {
        let val = val.value();
        block_context.set_local_var(key.to_string(), val.clone());
    }
    rc.push_block(block_context);
    Ok(())
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
) -> Result<String, TemplateRenderError> {
    let mut hbar = Handlebars::new();
    hbar.register_helper("set", Box::new(set_helper));
    hbar.register_helper("encode", Box::new(encode));
    hbar.render_template(template, &context)
}
