// Pure helpers of the planning dashboard (`REQ-TRS-VIS-027`); no DOM, so
// `frontend/test/planning-core.test.mjs` runs them in Node.
(function (root) {
  'use strict';

  /** `/ui/planning/board` with the active filters. */
  function boardUrl(who, showDone) {
    var q = [];
    if (who) { q.push('who=' + encodeURIComponent(who)); }
    if (showDone) { q.push('done=1'); }
    return '/ui/planning/board' + (q.length ? '?' + q.join('&') : '');
  }

  /** "12s", "3m", "2h", "5d" since an ISO-8601 instant; "" when unparseable; "now" for the future. */
  function age(iso, nowMs) {
    var t = Date.parse(iso);
    if (isNaN(t)) { return ''; }
    var s = Math.floor((nowMs - t) / 1000);
    if (s < 5) { return 'now'; }
    if (s < 60) { return s + 's'; }
    if (s < 3600) { return Math.floor(s / 60) + 'm'; }
    if (s < 86400) { return Math.floor(s / 3600) + 'h'; }
    return Math.floor(s / 86400) + 'd';
  }

  /** The select's options: "" first, then every known name once, sorted, keeping `current` even if it has no items. */
  function mergePeople(existing, fresh, current) {
    var set = {};
    existing.concat(fresh, current ? [current] : []).forEach(function (n) { if (n) { set[n] = true; } });
    return [''].concat(Object.keys(set).sort());
  }

  var api = { boardUrl: boardUrl, age: age, mergePeople: mergePeople };
  if (typeof module !== 'undefined' && module.exports) { module.exports = api; } else { root.PlanningCore = api; }
})(typeof window !== 'undefined' ? window : globalThis);
