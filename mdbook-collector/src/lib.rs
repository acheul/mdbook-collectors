use mdbook::book::{Book, BookItem};
use mdbook::errors::{Error, Result};
use mdbook::preprocess::{Preprocessor, PreprocessorContext};

use std::path::PathBuf;
use regex::Regex;
use serde_json::Value as JsValue;
use serde_json::Map as JsMap;

use std::fs::File;
use std::io::prelude::*;



#[derive(Clone, Copy, Default)]
enum DataType {
  #[default] Json,
  Yaml,
  Toml
}

impl DataType {
  fn new(str: &str) -> Self {
    match str.to_lowercase().as_str() {
      "yaml" | "yml" => {
        Self::Yaml
      },
      "toml" => Self::Toml, 
      _ => Self::default()
    }
  }
}


/// Config of the preprocessor
/// 
pub struct Config {
  input_type: DataType,
  /// the literal `collect` from `... <!-- collect {"json": "data"} --> ...`
  marker: String,
  /// save path of collected data
  save_path: PathBuf,
  /// Add post's title to the sub map or not
  add_title: bool
}

static DEFAULT_MARKER: &str = "collect";
static DEFAULT_SAVE_PATH: &str = "collect.json";


impl Config {
  fn new(preprocessor_name: &str, ctx: &PreprocessorContext) -> Result<Self> {

    let input_type = DataType::Json;
    let marker = String::from(DEFAULT_MARKER);
    let save_path = ctx.config.book.src.join(DEFAULT_SAVE_PATH);

    let mut config = Self {
      input_type,
      marker, 
      save_path,
      add_title: true
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

    if let Some(x) = get_value_to_str(cfg, "input_type") {
      config.input_type = DataType::new(x?.as_str());
    }

    if let Some(x) = get_value_to_str(cfg, "marker") {
      config.marker = x?;
    }

    if let Some(x) = get_value_to_str(cfg, "save_path") {
      config.save_path = ctx.config.book.src.join(x?.as_str());
    }

    if let Some(x) = cfg.get("add_title") {
      if let Some(x) = x.as_bool() {
        config.add_title = x;
      } else {
        return Err(Error::msg(format!("add_title {x:?} is not a valid boolean type")));
      }
    }

    // check out the regex syntax in advance.
    let _ = config.regex()?;

    Ok(config)
  }

  fn regex(&self) -> Result<Regex> {
    let marker = &self.marker;
    let re = format!("<!-- ?{}((?s).*?)-->", marker);
    if let Ok(re) = Regex::new(re.as_str()) {
      Ok(re)
    } else {
      Err(Error::msg(format!("marker {:?} has failed to be parsed into regular expression", marker)))
    }
  }

  /// Collect matched data and drain the content.
  fn collect_and_drain(&self, content: &mut String, name: &str) -> Option<JsMap<String, JsValue>> {

    let parse_to_json = move |ty: DataType, str: &str| {
      match ty {
        DataType::Json => {
          match serde_json::from_str::<JsMap<String, JsValue>>(str) {
            Ok(x) => Ok(x),
            Err(err) => Err(format!("While parsing Json data from post {name}, meets error: {}", err.to_string())),
          }
        },
        DataType::Yaml => {
          match serde_yaml::from_str::<JsMap<String, JsValue>>(str) {
            Ok(x) => Ok(x),
            Err(err) => Err(format!("While parsing Yaml data from post {name}, meets error: {}", err.to_string())),
          }
        },
        DataType::Toml => {
          match toml::from_str::<JsMap<String, JsValue>>(str) {
            Ok(x) => Ok(x),
            Err(err) => Err(format!("While parsing Toml data from post {name}, meets error: {}", err.to_string()))
          }
        }
      }
    };

    if let Some(cap) = self.regex().unwrap().captures(content.as_str()) {
      if let Some(match1) = cap.get(1) {
        
        let match0 = cap.get(0).unwrap();
        let (start, end) = (match0.start(), match0.end());

        match parse_to_json(self.input_type, match1.as_str()) {
          Ok(x) => {
            let _ = content.drain(start..end);
            return Some(x);
          },
          Err(msg) => { // Do not propagate this to Error
            log::debug!("{msg}");
          }
        }
      }
    }
    None
  }
}


/// The preprocessor struct
pub struct Collector;

impl Collector {
  pub fn new() -> Self { Self }
}

impl Preprocessor for Collector {

  fn name(&self) -> &str {
    "collector"
  }

  fn run(&self, ctx: &PreprocessorContext, mut book: Book) -> Result<Book> {
    
    log::trace!("Preprocessor Collector is working");
    
    let cfg = Config::new(self.name(), ctx)?;

    let mut map = JsMap::new();

    book.for_each_mut(|book| {
      if let BookItem::Chapter(chapter) = book {

        if let Some(path) = chapter.path.as_ref().map(|x| x.as_os_str().to_str()).flatten() {
          let name = chapter.name.to_string();
        
          if let Some(mut sub_map) = cfg.collect_and_drain(&mut chapter.content, &name) {
            if cfg.add_title {
              sub_map.insert("title".to_string(), JsValue::String(name.clone()));
            }
            map.insert(path.to_string(), JsValue::Object(sub_map));
          }
        }
      }
    });

    let len = map.len();

    if len>0 {
      let data = serde_json::to_string(&map).or(Err(Error::msg("Parsing into json has failed. Check out the Json syntax.")))?;
      let mut f = File::create(cfg.save_path.as_path()).unwrap();
      f.write_all(data.as_bytes())?;
      log::trace!("collected data of length {} has been saved in {:?}", len, cfg.save_path.as_path());
    } else {
      log::trace!("no post has collectable Json data: nothing saved.");
    }

    Ok(book)
  }
}