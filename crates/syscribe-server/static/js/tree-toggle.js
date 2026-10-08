// Expand/collapse of a package row in the model browser's tree.
//
// One handler owns the whole toggle (state, arrow and the children request).
// The markup used to split this between an inline `onclick` that flipped
// `data-open` and an htmx `hx-trigger="click[this.dataset.open!=='true']"`
// filter that read it: the inline handler ran first, so the filter always saw
// the state the click had just set — the first click flipped the arrow and
// loaded nothing, the second collapsed the row and *then* loaded its children
// (so the folder appeared open while its arrow said closed), the third did
// nothing. Here the request is made from the same place the state changes.
//
// `el` is the `.tree-toggle` span; its `data-url` is the `/ui/tree?parent=…`
// endpoint and the children container is the next sibling of its row.
(function () {
  var OPEN = '▼';
  var CLOSED = '▶';

  /** `ajax(method, url, {target, swap})` is `htmx.ajax`; injectable for tests. */
  function toggleTreeNode(el, ajax) {
    var children = el.parentElement.nextElementSibling;
    if (el.dataset.open === 'true') {
      el.dataset.open = 'false';
      el.textContent = CLOSED;
      children.innerHTML = '';
      return;
    }
    el.dataset.open = 'true';
    el.textContent = OPEN;
    var request = (ajax || window.htmx.ajax.bind(window.htmx))('GET', el.dataset.url, { target: children, swap: 'innerHTML' });
    // A failed load must not leave an open arrow over an empty folder.
    if (request && typeof request.catch === 'function') {
      request.catch(function () {
        el.dataset.open = 'false';
        el.textContent = CLOSED;
        children.textContent = 'Could not load this folder.';
      });
    }
  }

  if (typeof window !== 'undefined') {
    window.toggleTreeNode = function (el) { toggleTreeNode(el); };
  }
  if (typeof module !== 'undefined' && module.exports) {
    module.exports = { toggleTreeNode: toggleTreeNode, OPEN: OPEN, CLOSED: CLOSED };
  }
})();
