use handlebars::{Context, Handlebars, Helper, HelperResult, Output, RenderContext, RenderError};
use std::boxed::Box;

// implement via bare function
fn set_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    rc: &mut RenderContext,
    _out: &mut Output,
) -> HelperResult {
    for (key, val) in h.hash().iter() {
        let val = val.value();
        rc.set_local_var(key.to_owned(), val.clone());
    }

    Ok(())
}

pub fn get_template_renderer() -> Handlebars {
    let mut hbar = Handlebars::new();
    hbar.register_helper("set", Box::new(set_helper));
    hbar
}
