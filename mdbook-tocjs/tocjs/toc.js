/** Example
import { TocMaker } from "./lib/toc.js";

window.addEventListener("load", (e)=>{
  const toc_maker = new TocMaker(max_level=4);
  toc_maker.build_block("tock", use_number=false, block_title="titles");
  toc_maker.build_wing("tocw", use_number=false, use_title_name=false, root_title="Title", bar_unit_size=8, wing_left_margin=4);
});
*/


// TocMaker
//
// methods:
//  - build_array: initiated on construction.
//  - build_block: build the block type ToC
//  - build_wing: build the wing type ToC
class TocMaker {

  // constructor build the array of titles on initiation.
  // args:
  //  - max_level: Int // default=4 // Max level of titles
  constructor(max_level=4) {
    // parse args
    this.max_level = (max_level) ? max_level : 4;

    // build the array of format [[ith, level, number, link, name];]
    this.array = [];
    this.cur_max_level = 1;
    this.build_array();

    // Set up fieds for other methods.
    // They will be updated inside each methods.
    this.use_number = false;
    this.block_title = "Titles";
    this.use_title_name = false;
    this.root_title = "Title";
    this.bar_unit_size = 8;
    this.wing_left_margin = 4;
  }

  // getters
  get content_main() { 
    let content = document.getElementById("content");
    if (content) {
      return content.firstElementChild;
    } else {
      return;
    }
  }

  get page_wrapper() { return document.getElementById("page-wrapper"); }

  // nav-prev's width getter
  get nav_prev_width() {
    let page_wrapper = document.getElementById("page-wrapper");
    if (page_wrapper) {
      let nav_wrapper = page_wrapper.children.item(1);
      if (nav_wrapper) {
        let width = nav_wrapper.firstElementChild.getBoundingClientRect().width;
        return width;
      }
    }
    return;
  }


  // methods

  // Build the array
  build_array() {
    const main = this.content_main;
    if (main==null) {
      console.log("content's main is not found");
      return;
    }

    let i = 0;
    for (const part of main.children) {
      let tn = part.tagName.toLowerCase();
      let level = (tn=="h1") ? 1 : (tn=="h2") ? 2 : (tn=="h3") ? 3 : (tn=="h4") ? 4 : (tn=="h5") ? 5 : (tn=="h6") ? 6 : null; 
      if (level != null) {
        if (level<=this.max_level) {

          if (level>this.cur_max_level) {
            this.cur_max_level = level;
          }

          // add dataset-ith for the part
          part.dataset.ith = `${i}`;

          let a = part.firstElementChild;
          let link = "#"+a.href.split("#")[1];
          let name = a.textContent;
          
          let len = this.array.length;
          let number = (len==0) ? null : (this.array[len-1][1]==level) ? this.array[len-1][2]+1 : 1;
          this.array.push([i, level, number, link, name]);
          i += 1;
        }
      }
    }
  }

  // Build ToC block
  //
  // block
  //  div("tock-box")
  //    div("tock-cell tock-root")
  //      text{block_title}
  //    div("tock-cell tock-each")
  //      a("tock-a tock-padl-{level}")
  //        span("tock-a-num")
  //        span{name}
  //
  // args:
  //  - block_id: marker node's id
  //  /* Below args can be passed with html dataset property */
  //  - use_number: bool // default=false
  //  - block_title: // Option[String] // default="Titles" // Top title of the block //
  build_block(block_id, use_number, block_title) {

    if (this.array.length==0) {
      return;
    }

    const block = document.getElementById(block_id);
    if (block==null) { console.log("The ToC Block's id is not valid."); return; }


    // parse args
    // use_number -- from args or dataset or false
    this.use_number = (use_number) ? use_number : block.dataset.use_number=="true";

    // block title -- from args or dataset or "Titles"
    this.block_title = (block_title) ? block_title : (block.dataset.block_title) ? block.dataset.block_title : "Titles";


    // building
    let box = document.createElement("div");
    box.classList.add("tock-box");

    let text = document.createTextNode(this.block_title);
    let cell = document.createElement("div");        
    cell.classList.add("tock-cell", "tock-root");
    cell.appendChild(text);
    box.appendChild(cell);
    
    for (const [ith, level, number, link, name] of this.array) {
      // skip root title for Block.
      if (ith>0) {
        let a = document.createElement("a");
        a.href = link;
        a.title = name;
        a.dataset.ith = `${ith}`;
        a.classList.add("tock-a", `tock-padl-${level}`);

        if (this.use_number && number!=null) {
          let text = document.createTextNode(`${number}`);
          let span = document.createElement("span");
          span.classList.add("tock-a-num");
          span.appendChild(text);
          a.appendChild(span);
        }
        let text = document.createTextNode(name);
        let span = document.createElement("span");
        span.appendChild(text);
        a.appendChild(span);
        
        let cell = document.createElement("div");
        cell.classList.add("tock-cell", "tock-each");
        cell.appendChild(a);
        box.appendChild(cell);
      }
    }
    block.appendChild(box);
  }


  // Build ToC Wing
  //
  // block
  //  div("tocw-box")
  //    div("tocw-cell tocw-each")
  //      a("tocw-a")
  //        div("tocw-a-bar")
  //        div("tocw-a-title")
  //          span("tocw-a-num")
  //          span{name}
  //
  // args:
  //  - wing_id: marker node's id
  //  /* Below args can be passed with html dataset property */
  //  - use_number: bool // default=false
  //  - use_title_name: bool // default=false // Use root's title name or not. If not, use "Title".
  //  - root_title: Option[String] // default="Title" // if use_title_name==false, use this one as root's title.
  //  - bar_unit_size: int // default=8 // unit size of the bar
  //  - wing_left_margin: int // default=4 // left-margin of the wing
  build_wing(wing_id, use_number, use_title_name, root_title, bar_unit_size, wing_left_margin) {

    if (this.array.length==0) {
      return;
    }
    
    // set page-wrapper's pos relative (the wing's wrapper will be absolute to it.)
    this.page_wrapper.style.position = "relative";

    const wing = document.getElementById(wing_id);
    if (wing==null) { console.log("The ToC Wing's id is not valid."); return; }


    // parse args
    // use_number -- from args or dataset or false
    this.use_number = (use_number) ? use_number : wing.dataset.use_number=="true";

    // root title
    this.use_title_name = (use_title_name) ? use_title_name : wing.dataset.use_title_name != "false";
    this.root_title = (this.use_title_name) ? null : (root_title) ? root_title : "Title";

    this.bar_unit_size = (bar_unit_size) ? bar_unit_size : (wing.dataset.bar_unit_size) ? parseInt(wing.dataset.bar_unit_size) : 8;
    this.wing_left_margin = (wing_left_margin) ? wing_left_margin : (wing.dataset.wing_left_margin) ? parseInt(wing.dataset.wing_left_margin) : 4;


    // building
    let box = document.createElement("div");
    box.classList.add("tocw-box");

    for (const [ith, level, number, link, name] of this.array) {

      let a = document.createElement("a");
      a.href = link;
      a.title = name;
      a.dataset.ith = `${ith}`;
      a.classList.add("tocw-a");

      // title div
      let title = document.createElement("div");
      title.dataset.ith = `${ith}`;
      title.classList.add("opacity0", "tocw-a-title", `tocw-padl-${level}`);

      if (ith>0 && this.use_number && number!=null) {
        let text = document.createTextNode(`${number}`);
        let span = document.createElement("span");
        span.classList.add("tocw-a-num");
        span.appendChild(text);
        title.appendChild(span);
      }
      let name_ = (ith==0 && this.root_title!=null) ? this.root_title : name;
      let text = document.createTextNode(name_);
      let span = document.createElement("span");
      span.appendChild(text);
      title.appendChild(span);

      // bar div
      let bar = document.createElement("div");
      bar.dataset.ith = `${ith}`;
      bar.classList.add("tocw-a-bar");
      let size = (this.cur_max_level-level+1) * this.bar_unit_size;
      bar.style.width = `${size}px`;

      a.appendChild(bar);
      a.appendChild(title);

      let cell = document.createElement("div");
      cell.classList.add("tocw-cell", "tocw-each");
      cell.appendChild(a);
      box.appendChild(cell);
    }
    // box < wrap < page-wrapper
    let wrap = document.createElement("div");
    wrap.classList.add("tocw-wrap");
    wrap.style.maringLeft = `${this.wing_left_margin}px`;
    wrap.appendChild(box);

    let page_wrapper = this.page_wrapper;
    page_wrapper.appendChild(wrap);


    // set handlers

    this._set_wing_wrap_pos(wrap);
    this._focus_current_title(box);
    
    window.addEventListener("resize", (e)=>{
      this._set_wing_wrap_pos(wrap);
    });
    window.addEventListener("scroll", (e)=>{
      this._set_wing_wrap_pos(wrap);
      this._focus_current_title(box);
    });
    let left_bttn = document.getElementsByClassName("left-buttons")[0].firstElementChild;
    left_bttn.addEventListener("click", (e)=>{
      this._set_wing_wrap_pos(wrap);
    });


    // box - enter/leave event handlers
    // -> flip opacity btw title & bar 
    box.addEventListener("mouseenter", (e)=>{
      for (const title of box.getElementsByClassName("tocw-a-title")) {
        title.classList.remove("opacity0");
      }
      for (const bar of box.getElementsByClassName("tocw-a-bar")) {
        bar.classList.add("opacity0");
      }
    });

    box.addEventListener("mouseleave", (e)=>{
      for (const title of box.getElementsByClassName("tocw-a-title")) {
        title.classList.add("opacity0");
      }
      for (const bar of box.getElementsByClassName("tocw-a-bar")) {
        bar.classList.remove("opacity0");
      }
    });
  }


  // helpers

  // set wing-wrap's positon
  _set_wing_wrap_pos(wrap) {
    
    let height = window.innerHeight;

    // check visiblity
    let content = document.getElementById("content");
    let main = content.firstElementChild;
    let gap = content.getBoundingClientRect().width - main.getBoundingClientRect().width;
    gap *= 0.5;

    let nav_prev_width = this.nav_prev_width;
    if (nav_prev_width) {
      gap -= nav_prev_width;
    }

    if (gap>this.cur_max_level*this.bar_unit_size + this.wing_left_margin) {

      let left = (nav_prev_width) ? nav_prev_width : 0;
      let top = window.scrollY;

      wrap.style.height = `${height}px`;
      wrap.style.width = `${gap}px`;
      wrap.style.top = `${top}px`;
      wrap.style.left = `${left}px`;
      wrap.style.visibility = "visible";
    } else {
      wrap.style.visibility = "hidden";
    }
  }

  // make current title focused
  _focus_current_title(box) {

    let i;
    let content_main = this.content_main;
    if (content_main==null) {
      console.log("content's main is not found.");
      return;
    }

    for (const part of content_main.children) {
      if (part.getBoundingClientRect().top > 0) {
        let i_ = part.dataset.ith;
        if (i_!=null) {
          i = i_;
          break;
        }
      }
    }

    for (const title of box.getElementsByClassName("tocw-a-title")) {
      if (title.dataset.ith==i && i!=null) {
        title.classList.add("tocw-a-title-cur");
      } else {
        title.classList.remove("tocw-a-title-cur");
      }
    }
    for (const bar of box.getElementsByClassName("tocw-a-bar")) {
      if (bar.dataset.ith==i && i!=null) {
        bar.classList.add("tocw-a-bar-cur");
      } else {
        bar.classList.remove("tocw-a-bar-cur");
      }
    }
  }
}