use mdbook::book::{Book, BookItem};
use mdbook::errors::{Error, Result};
use mdbook::preprocess::{Preprocessor, PreprocessorContext};

use std::path::PathBuf;
use regex::Regex;
use hashbrown::HashMap;

use std::fs::File;
use std::io::prelude::*;


pub struct Config {
  /// the literal `tags` from `... <!-- tags: hype; boy; --> ...`
  marker: String,
  /// save path of tag2posts data file
  tag2posts_path: PathBuf,
  /// save path of post2tags data file
  post2tags_path: PathBuf,
  /// Split pattern to slice tags literal
  split: String
}

static DEFAULT_MARKER: &str = "tags";
static DEFAULT_TAG2POSTS_PATH: &str = "tag2posts.json";
static DEFAULT_POST2TAGS_PATH: &str = "post2tags.json";
static DEFAULT_SPLIT: &str = ";";

impl Config {
  fn new(preprocessor_name: &str, ctx: &PreprocessorContext) -> Result<Self> {

    let marker = String::from(DEFAULT_MARKER);
    let tag2posts_path = ctx.config.book.src.join(DEFAULT_TAG2POSTS_PATH);
    let post2tags_path = ctx.config.book.src.join(DEFAULT_POST2TAGS_PATH);

    let mut config = Self {
      marker, 
      tag2posts_path,
      post2tags_path,
      split: String::from(DEFAULT_SPLIT)
    };

    let Some(cfg) = ctx.config.get_preprocessor(preprocessor_name) else {
      return Ok(config)
    };

    let get_value_to_str = |cfg: &toml::map::Map<String, toml::value::Value>, key: &str| {
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
    };

    if let Some(x) = get_value_to_str(cfg, "marker") {
      config.marker = x?;
    }

    if let Some(x) = get_value_to_str(cfg, "tag2posts_path") {
      config.tag2posts_path = ctx.config.book.src.join(x?.as_str());
    }

    if let Some(x) = get_value_to_str(cfg, "post2tags_path") {
      config.post2tags_path = ctx.config.book.src.join(x?.as_str());
    }

    if let Some(x) = get_value_to_str(cfg, "split") {
      config.split = x?;
    }

    // check out the regex syntax in advance.
    let _ = config.regex()?;

    Ok(config)
  }

  fn regex(&self) -> Result<Regex> {
    let marker = &self.marker;
    let re = format!("<!-- ?{}:?((?s).*?)-->", marker);
    if let Ok(re) = Regex::new(re.as_str()) {
      Ok(re)
    } else {
      Err(Error::msg(format!("marker {:?} has failed to be parsed into regular expression", marker)))
    }
  }

  /// Collect matched data and drain the content.
  fn collect_and_drain(
    &self,
    content: &mut String,
    name: &str,
    path: String,
    tag2posts: &mut HashMap<String, Vec<(String, String)>>,
    post2tags: &mut HashMap<String, Vec<String>>
  ) -> () {

    let parse_to_tags = |str: &str| {
      str.trim().split(&self.split).into_iter().filter_map(|x| {
        let x = x.trim();
        if x.len()>0 {
          Some(x.to_string())
        } else {
          None
        }
      }).collect::<Vec<_>>()
    };

    let (mut start, mut end) = (None, None);

    if let Some(cap) = self.regex().unwrap().captures(content.as_str()) {
      if let Some(match1) = cap.get(1) {
        
        let match0 = cap.get(0).unwrap();
        start.replace(match0.start());
        end.replace(match0.end());

        let tags = parse_to_tags(match1.as_str());
        post2tags.insert(path.clone(), tags.clone());

        for tag in tags.into_iter() {
          let post_ = (name.to_string(), path.clone());
          tag2posts.entry(tag)
            .and_modify(|list| { list.push(post_.clone()); })
            .or_insert(vec![post_]);
        }
      }
    }

    // drain
    if let Some(start) = start {
      if let Some(end) = end {
        let _ = content.drain(start..end);
      }
    }
  }
}



pub struct Tagger;

impl Tagger {
  pub fn new() -> Self { Self }
}

impl Preprocessor for Tagger {
  
  fn name(&self) -> &str {
    "tagger"
  }

  fn run(&self, ctx: &PreprocessorContext, mut book: Book) -> Result<Book> {
      
    log::trace!("Preprocessor Collector is working");
    
    let cfg = Config::new(self.name(), ctx)?;

    let mut tag2posts = HashMap::new();
    let mut post2tags = HashMap::new();

    book.for_each_mut(|book: &mut BookItem| {
      if let BookItem::Chapter(chapter) = book {

        if let Some(path) = chapter.path.as_ref().map(|x| x.as_os_str().to_str()).flatten() {
          let name = chapter.name.to_string();
          
          cfg.collect_and_drain(&mut chapter.content, &name, path.to_string(), &mut tag2posts, &mut post2tags);
        }
      }
    });

    if tag2posts.len()>0 {
      let data = serde_json::to_string(&tag2posts).unwrap();
      let mut f = File::create(cfg.tag2posts_path.as_path()).unwrap();
      f.write_all(data.as_bytes())?;
    }
    if post2tags.len()>0 {
      let data = serde_json::to_string(&post2tags).unwrap();
      let mut f = File::create(cfg.post2tags_path.as_path()).unwrap();
      f.write_all(data.as_bytes())?;
    }

    Ok(book)
  }
}