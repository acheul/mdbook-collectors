use mdbook::book::Book;
use mdbook::errors::{Error, Result};
use mdbook::preprocess::{Preprocessor, PreprocessorContext};

use std::path::{Path, PathBuf};
use std::fs::{self, File};
use std::io::prelude::*;

mod blobs;
use blobs::*;


pub struct Config {
  save_dir: PathBuf,
  block_marker_id: String,
  wing_marker_id: String,
  theme_dir: PathBuf,
  base_url: String,
}

static DEFAULT_SAVE_DIR: &str = "lib";
static DEFAULT_BLOCK_MARKER_ID: &str = "tock";
static DEFAULT_WING_MARKER_ID: &str = "tocw";
static DEFAULT_THEME_DIR: &str = "theme";
static DEFAULT_BASE_URL: &str = "/";


fn get_value_to_str(cfg: &toml::map::Map<String, toml::value::Value>, key: &str) -> Option<Result<String>> {
  if let Some(x) = cfg.get(key) {
    let res = if let Some(x) = x.as_str() {
      Ok(x.to_string())
    } else {
      Err(Error::msg(format!("{key} {x:?} is not a valid string")))
    };
    Some(res)
  } else {
    None
  }
}


impl Config {
  fn new(preprocessor_name: &str, ctx: &PreprocessorContext) -> Result<Self> {

    let mut config = Self {
      save_dir: ctx.config.book.src.join(DEFAULT_SAVE_DIR),
      block_marker_id: String::from(DEFAULT_BLOCK_MARKER_ID),
      wing_marker_id: String::from(DEFAULT_WING_MARKER_ID),
      theme_dir: Path::new(DEFAULT_THEME_DIR).to_path_buf(), // This one is not under /src
      base_url: String::from(DEFAULT_BASE_URL)
    };

    let Some(cfg) = ctx.config.get_preprocessor(preprocessor_name) else {
      return Ok(config)
    };

    if let Some(x) = get_value_to_str(cfg, "save_dir") {
      config.save_dir = ctx.config.book.src.join(x?.as_str());
    }
    if let Some(x) = get_value_to_str(cfg, "block_marker_id") {
      config.block_marker_id = x?;
    }
    if let Some(x) = get_value_to_str(cfg, "wing_marker_id") {
      config.wing_marker_id = x?;
    }
    if let Some(x) = get_value_to_str(cfg, "theme_dir") {
      config.theme_dir = Path::new(x?.as_str()).to_path_buf(); // This one is not under /src
    }
    if let Some(x) = get_value_to_str(cfg, "base_url") {
      config.base_url = x?;
    }

    if !config.save_dir.exists() {
      fs::create_dir(config.save_dir.as_path())?;
    }
    if !config.theme_dir.exists() {
      fs::create_dir(config.theme_dir.as_path())?;
    }

    Ok(config)
  }

  fn format_js(&self, literal: &str) -> String {
    format!(r#"
      {}
      window.addEventListener("load", (e)=>{{
        const toc_maker = new TocMaker();
        toc_maker.build_block("{}");
        toc_maker.build_wing("{}");
      }});
      "#,
      literal,
      self.block_marker_id,
      self.wing_marker_id
    )
  }
}



pub struct TocJsMaker;

impl TocJsMaker {
  pub fn new() -> Self { Self }
}


impl Preprocessor for TocJsMaker {

  fn name(&self) -> &str {
    "tocjs"
  }

  fn run(&self, ctx: &PreprocessorContext, book: Book) -> Result<Book> {
        
    let cfg = Config::new(self.name(), ctx)?;

    let mut f = File::create(cfg.save_dir.join("toc.css")).unwrap();
    f.write_all(TOC_CSS.as_bytes())?;

    let mut f = File::create(cfg.save_dir.join("toc.js")).unwrap();
    f.write_all(cfg.format_js(TOC_JS).as_bytes())?;

    // make theme/head.hbs
    let head_hbs_literals = [
      format!(r#"<link rel="stylesheet" href="{}lib/toc.css">"#, cfg.base_url),
      format!(r#"<script src="{}lib/toc.js"></script>"#, cfg.base_url)
    ];
      
    let head_hbs_path = cfg.theme_dir.join("head.hbs");

    match head_hbs_path.exists() {
      true => {
        let mut f = std::fs::OpenOptions::new()
          .read(true).write(true).append(true).open(head_hbs_path.as_path()).unwrap();

        let mut buf: String = String::new();
        f.read_to_string(&mut buf)?;

        for literal in head_hbs_literals {
          if !buf.contains(&literal) {
            f.write_all(literal.as_bytes())?;
          }
        }
      },
      false => {
        let mut f = File::create(head_hbs_path.as_path()).unwrap();
        f.write_all(head_hbs_literals.join("").as_bytes())?;
      }
    }

    Ok(book)
  }
}