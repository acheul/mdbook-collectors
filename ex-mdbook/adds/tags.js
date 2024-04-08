window.addEventListener("load", (e)=>{

  let mark = document.getElementById("Tags");
  if (mark) {
    fetch("tag2posts.json").then((res)=>res.json()).then((data)=>{
      for (const [tag_, post_] of Object.entries(data)) {
  
        let div = document.createElement("div");
        div.classList.add("cell-tag");

        let tag_name = document.createTextNode(tag_);
        let tag = document.createElement("div");
        tag.appendChild(tag_name);
        tag.classList.add("tag-name");

        div.appendChild(tag);
  
        for (const [name_, path_] of post_) {
          let name = document.createTextNode(name_);
          let path = (path_.slice(-3)=='.md') ? path_.slice(0, -3)+".html" : path_;
          
          let a = document.createElement("a");
          a.href = path.replace(".md", ".html");
          a.appendChild(name);
          a.classList.add("tag-post");
          
          div.appendChild(a);
        }

        mark.appendChild(div);
      }
    });
  }
});