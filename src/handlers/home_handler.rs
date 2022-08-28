use actix_web::{get, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use std::{fs, io::Error}; 
use ignore::WalkBuilder; 

#[derive(Serialize, Deserialize, Debug)]
pub struct Frontmatter {
    title: String,
    file_name: String,
    description: String,
    posted: String,
    tags: Vec<String>,
    author: String,
    estimated_reading_time: u32,
    order: u32,
}

fn find_all_frontmatters() -> Result<Vec<Frontmatter>, std::io::Error> {
    let mut t = ignore::types::TypesBuilder::new();
    t.add_defaults();
    let toml = match t.select("toml").build() {
	  Ok(t)=> t,
	  Err(e)=> {
		println!("{:}", e);
		return Err(Error::new(std::io::ErrorKind::Other,
		"could not build toml file type matcher"))
	  }
    };
    
    let file_walker = WalkBuilder::new("./posts").types(toml).build();

    let mut frontmatters = Vec::new();
    for frontmatter in file_walker {
        match frontmatter {
	      Ok(fm) => {
		    if fm.path().is_file() {
		        let fm_content = fs::read_to_string(fm.path())?;
			  let frontmatter: Frontmatter = toml::from_str(&fm_content)?;
			  
			  frontmatters.push(frontmatter);
		    }
		}
		Err(e) => {
		    println!("{:}", e); // we're just going to print the error for now
		    return Err(Error::new(std::io::ErrorKind::NotFound, "could not locate frontmatter"))
		}
	  }
    }
    
    Ok(frontmatters)
}

#[get("/")]
pub async fn index(templates: web::Data<tera::Tera>) -> impl Responder {
   let mut context = tera::Context::new();
   
   let frontmatters = vec![Frontmatter{
	tags: vec!["Rusty".to_string(), "Test".to_string()],
	title: "Test posts".to_string(),
	file_name: "test_posts.md".to_string(),
	description: "Just testing out the system".to_string(),
	posted: "2022-08-09".to_string(),
	author: "MBvisti".to_string(),
	estimated_reading_time: 12,
	order: 1,
   }];
   
   context.insert("posts", &frontmatters);
   
   match templates.render("home.html", &context) {
	Ok(s) => HttpResponse::Ok().content_type("text/html").body(s),
	Err(e) => {
		println!("{:?}", e);
		HttpResponse::InternalServerError()
			.content_type("text/html")
			.body("<p>Something went wrong!</p>")
	}
   }
}
