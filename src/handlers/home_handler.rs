use actix_web::{get, web, HttpResponse, Responder};
use ignore::WalkBuilder;
use std::{fs, io::Error};
use crate::constants::CONTENT_TYPE_TEXT_HTML;
use crate::models::frontmatter::Frontmatter;


fn get_all_file_contents_in_folder(folder: &str, file_type: &str) -> Result<Vec<Frontmatter>, Error> {
    let mut type_builder = ignore::types::TypesBuilder::new();
    type_builder.add_defaults();
    let file_type_matcher = type_builder.select(&file_type).build()
        .map_err(|err| {
            Error::new(std::io::ErrorKind::Other, format!("Could not build {} file type matcher. Error: {:?}", &file_type, err))
        })?;

    let post_folder_toml_files = WalkBuilder::new(folder).types(file_type_matcher).build();

    let mut frontmatters = Vec::new();
    for toml_file in post_folder_toml_files {
        let toml_file = toml_file.map_err(|err|{
            Error::new(
                std::io::ErrorKind::NotFound,
                format!("Could not locate file with format {}. Error: {:?}", &file_type, err),
            )
        })?;

        if toml_file.path().is_file() {
            let fm_content = fs::read_to_string(toml_file.path())?;
            let frontmatter: Frontmatter = toml::from_str(&fm_content)?;

            frontmatters.push(frontmatter);
        }
    }

    Ok(frontmatters)
}

#[get("/")]
pub async fn index(templates: web::Data<tera::Tera>) -> impl Responder {

    if let Ok(mut frontmatters) = get_all_file_contents_in_folder("./posts", "toml") {
        let mut context = tera::Context::new();
        frontmatters.sort_by_key(|p| std::cmp::Reverse(p.order()));
        context.insert("posts", &frontmatters);

        if let Ok(template_render) = templates.render("home.html", &context) {
            return HttpResponse::Ok().content_type(CONTENT_TYPE_TEXT_HTML).body(template_render);
        }
    }
    return HttpResponse::InternalServerError()
        .content_type(CONTENT_TYPE_TEXT_HTML)
        .body("<p>Something went wrong!</p>");
}
