# mdbook-collector

A [mdbook preprocessor](https://rust-lang.github.io/mdBook/format/configuration/preprocessors.html) which collects json/yaml/toml data from each post and builds a json file of it


## Install
```
cargo install mdbook-collector
```

## Use
In a markdown file of a post, include:
```html
<!-- collect
{
  "data": {
    "type": "json",
    "keywords": ["json", "collect"]
  }
}
-->
```
Then the preprocessor will parse each post and build a consolidated json file, which is a map whose key is each post's name and whose values are (1)post's path and (2)sub-map of parsed data:
```json
{
  "post's name": {
    "path": "posts/path.md",
    "data" {
      "type": "json",
      "keywords": ["json", "collect"]
    }
  }
}
```
This json file can be used in your additional .js files, for example.

Make sure that each post's raw json data should be able to be parsed into "map" structure. For example, below will not be parsed and thus be ignored:
```html
<!-- collect
  "data": {
    "type": "json",
    "keywords": ["json", "collect"]
  }
-->
```

If Json's strictness and no-comment-ness bothers you, use yaml or toml option.

## Configuration
Configuration example in `book.toml`. Below is default setting:
```toml
[preprocessor.collector]
input_type = "json"
marker = "collect"
save_path = "collect.json"
path_key = "path"
```

### input_type
The output file is always in .json format. However you can select to-be-parsed input data's format among `json`, `yaml`, or `toml`.

For example, using `toml`:
```
<!-- collect
  [data]
  type = "toml"
  keywords=["toml", "collect"] # any comment
-->
```

The preprocessor depends on `serde_json` for json option, `serde_yaml` for yaml, and `toml` for toml. The `yaml` and `toml` options parse not only oneselves' format but also json format, as their dependant crates work in such ways.

### marker
The preprocessor tries to parse any literal located between the first `<!-- collect` and `-->` in each post's source file. (`<!--collect` instead of `<!-- collect` is ok.)

You can change the marking literal. Default is `collect`.

### save_path
Newly created json file will be saved at a path which combines book's `src` directory and the given `save_path`.

It must be saved under the `src` directory to be auto copied into build-dir.

> 🪧 Mind that the `mdbook serve` watches `src` directory. Once you start `mdbook serve` command then change any contents under `src` directory, the collector will rebuild the json file and this leads to a repeat loop of watch and serve. To prevent this, make a **.gitignore** file at the book's root directory and add the to-be-built json file's name.


### path_key
The collector makes a json map whose key is each post's name and whose value is a sub map parsed from each post's source file.

The collector insert one more information to the sub map: each post's path. Hereby the inserted value is the actual path and its key is designated from the `path_key` configuration. Default is `path`.

Collected paths can be used to make href link in an additional js file, for example.