use std::{io::Error, fs};

use actix_web::{web, get, Responder, HttpResponse};
use pulldown_cmark::{Options, Parser, html};
use crate::constants::CONTENT_TYPE_TEXT_HTML;
use crate::models::frontmatter::Frontmatter;


fn extract_markdown(post_name: &str) -> Result<String, Error> {
	fs::read_to_string(format!("./posts/{}/post.md", post_name))
}

fn extract_frontmatter(post_name: &str) -> Result<Frontmatter, Error> {
	let frontmatter_input = fs::read_to_string(format!("./posts/{}/post_frontmatter.toml", post_name))?;

	toml::from_str(&frontmatter_input)
		.map_err(|_err| {
			Error::new(std::io::ErrorKind::Other, "Not a valid TOML file.")
		})
}

#[get("/posts/{post_name}")]
pub async fn post(
	tmpl: web::Data<tera::Tera>,
	post_name: web::Path<String>,
) -> impl Responder {
	if let (Ok(markdown_input), Ok(frontmatter)) = (extract_markdown(&post_name), extract_frontmatter(&post_name)) {
		let mut html_output = String::new();
		let parser = Parser::new_ext(&markdown_input, Options::empty());
		html::push_html(&mut html_output, parser);

		let mut context = tera::Context::new();
		context.insert("post", &html_output);
		context.insert("meta_data", &frontmatter);

		if let Ok(tmpl_render) = tmpl.render("post.html", &context) {
			return HttpResponse::Ok().content_type(CONTENT_TYPE_TEXT_HTML).body(tmpl_render);
		}
	}
	HttpResponse::NotFound()
		.content_type(CONTENT_TYPE_TEXT_HTML)
		.body("<p>Could not find post - sorry!</p>")
}
