/**
 * StreamKit shared utilities.
 *
 * Loaded by every tool via:
 *   <script src="../../../assets/js/util.js"></script>
 *
 * Exposes a `StreamKit` namespace and a small set of legacy-friendly globals
 * (esc, escHtml, uid, copyOBSLink) so older inline code keeps working.
 */
(function (global) {
  'use strict';

  const StreamKit = (global.StreamKit = global.StreamKit || {});

  /** HTML-escape a value for safe interpolation. Covers all 5 sensitive chars. */
  StreamKit.esc = function (s) {
    return String(s)
      .replace(/&/g, '&amp;')
      .replace(/"/g, '&quot;')
      .replace(/'/g, '&#39;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;');
  };

  /** Short unique-ish ID for in-memory list items. NOT cryptographic. */
  StreamKit.uid = function () {
    return Date.now().toString(36) + Math.random().toString(36).slice(2, 5);
  };

  /**
   * Copy this page's OBS overlay URL to the clipboard and animate the trigger
   * button. Pass either the button's ID string or the element itself; defaults
   * to `#copyOBSBtn` if no argument is given.
   */
  StreamKit.copyOBSLink = function (btn) {
    const url = window.location.href.replace('index.html', 'overlay.html');
    navigator.clipboard.writeText(url).then(function () {
      const el = typeof btn === 'string' ? document.getElementById(btn)
              : btn instanceof HTMLElement ? btn
              : document.getElementById('copyOBSBtn');
      if (!el) return;
      const orig = el.dataset.origLabel || el.textContent;
      el.dataset.origLabel = orig;
      el.textContent = '✓ Copied!';
      el.classList.add('copied');
      setTimeout(function () {
        el.textContent = orig;
        el.classList.remove('copied');
      }, 2000);
    });
  };

  /**
   * Shared font catalog used by every tool's font picker.
   *   { id, label, css, license }
   * License values:
   *   'System'       → installed with the OS, free to use
   *   'Free · OFL'   → bundled, OFL-licensed (commercial use permitted)
   *   'Attribution'  → bundled, attribution required
   */
  StreamKit.FONTS = [
    { id: 'system',    label: 'Default',    css: "'Segoe UI', system-ui, sans-serif",                    license: 'System' },
    { id: 'impact',    label: 'Impact',     css: "Impact, 'Arial Narrow', sans-serif",                   license: 'System' },
    { id: 'bebas',     label: 'Bebas Neue', css: "'Bebas Neue', sans-serif",                             license: 'Free · OFL' },
    { id: 'mono',      label: 'Mono',       css: "'Consolas', 'Courier New', monospace",                 license: 'System' },
    { id: 'trebuchet', label: 'Trebuchet',  css: "'Trebuchet MS', sans-serif",                           license: 'System' },
    { id: 'georgia',   label: 'Georgia',    css: "Georgia, serif",                                       license: 'System' },
    { id: 'franklin',  label: 'Franklin',   css: "'Franklin Gothic Medium', 'Arial Narrow', sans-serif", license: 'System' },
    { id: 'rounded',   label: 'Rounded',    css: "'Arial Rounded MT Bold', 'Arial', sans-serif",         license: 'System' },
  ];

  /** Derived map (id → css) used by overlay pages. */
  StreamKit.FONT_CSS = Object.fromEntries(
    StreamKit.FONTS.map(function (f) { return [f.id, f.css]; })
  );

  /** Map a license value to its badge CSS class. */
  StreamKit.licenseClass = function (license) {
    if (license === 'Free · OFL' || license === 'Commercial') return 'lic-commercial';
    if (license === 'Attribution') return 'lic-attribution';
    return 'lic-system';
  };

  // ── Legacy globals — keeps existing tool code working without per-call edits ─
  global.esc         = StreamKit.esc;
  global.escHtml     = StreamKit.esc;
  global.uid         = StreamKit.uid;
  global.copyOBSLink = StreamKit.copyOBSLink;
})(window);
