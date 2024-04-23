# mdbook-tagger

[![Crates.io](https://img.shields.io/crates/v/mdbook-tagger)](https://crates.io/crates/mdbook-tagger)

A [mdbook preprocessor](https://rust-lang.github.io/mdBook/format/configuration/preprocessors.html) which collects and builds tag data in Json format.

## Install
```
cargo install mdbook-tagger
```

## Use (v0.2.0)
In a .md file of a post, include:
```html
<!-- tags: computer; book -->
```

The preprocessor parses each post's tag data and then builds two consolidated json files:

### (1) Tag to Posts
```json
{
  "tag's name": [
    ["post's name", "post's path"],
    ["other-post's name", "other-post's path"],
    ...
  ],
  ...
}
```

### (2) Post to Tags
```json
{
  "post's url": ["tag's name", "other tag's name", ...],
  ...
}
```

* Mind that in the v.0.1.0, the key of this map was post's name(title). From v.0.2.0, each post's url path is the key of map.


## Configuration
Default setting (in `book.toml`):
```toml
[preprocessor.tagger]
marker = "tags"
split = ";"
tag2posts_path = "tag2posts.json"
post2tags_path = "post2tags.json"
```

### marker
By default, the preprocessor tries to parse any literal located between the first `<!-- tags` and `-->` in each post's source file. (`<!--tags` instead of `<!-- tags` is ok.)

### split
```html
<!-- tags: computer; book; duck; -->
```
This will be parsed into tags of ["computer", "book", "duck"].

You can change the default seperator pattern ";".

If split = ",",
```html
<!-- tags: computer, book, duck -->
```
will be parsed into tags of ["computer", "book", "duck"].

### tag2posts_path, post2tags_path
The path where newly built json files will be saved.

Each json file will be saved at a path which combines book's `src` directory and the given `tag2posts_path` and `post2tags_path`.

> 🪧 Mind that the `mdbook serve` watches `src` directory. Once you start `mdbook serve` command then change any contents under `src` directory, the preprocessor will rebuild the json file and this leads to a repeat loop of watch and serve. To prevent this, make a **.gitignore** file at the book's root directory and add the to-be-built json file's name.