// Syscribe static HTML export — offline client-side search.
//
// Reads the element index embedded inline in index.html as
// `window.SEARCH_INDEX` (an array of {qname,id,type,name,url}). This avoids
// fetch(), which browsers block for file:// pages, so the site searches fully
// offline. Filters on a text box and renders matching result links.
(function () {
  "use strict";

  function ready(fn) {
    if (document.readyState !== "loading") {
      fn();
    } else {
      document.addEventListener("DOMContentLoaded", fn);
    }
  }

  ready(function () {
    var index = window.SEARCH_INDEX || [];
    var box = document.getElementById("search-box");
    var results = document.getElementById("search-results");
    if (!box || !results) {
      return;
    }

    function render(items) {
      results.innerHTML = "";
      items.forEach(function (e) {
        var li = document.createElement("li");
        var a = document.createElement("a");
        a.href = e.url;
        a.textContent = e.name || e.qname;
        var qn = document.createElement("span");
        qn.className = "qn";
        qn.textContent = " " + (e.type || "") + " · " + e.qname;
        li.appendChild(a);
        li.appendChild(qn);
        results.appendChild(li);
      });
    }

    function filter() {
      var q = box.value.trim().toLowerCase();
      if (q === "") {
        render(index);
        return;
      }
      var matches = index.filter(function (e) {
        var hay = [e.qname, e.id, e.name, e.type]
          .filter(Boolean)
          .join(" ")
          .toLowerCase();
        return hay.indexOf(q) !== -1;
      });
      render(matches);
    }

    box.addEventListener("input", filter);
    render(index);
  });
})();
