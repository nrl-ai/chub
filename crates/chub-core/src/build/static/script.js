// Theme toggle — respects OS preference, persists to localStorage
(function initTheme() {
  var saved = localStorage.getItem('chub-theme');
  if (saved) {
    document.documentElement.setAttribute('data-theme', saved);
  } else if (window.matchMedia && window.matchMedia('(prefers-color-scheme: light)').matches) {
    document.documentElement.setAttribute('data-theme', 'light');
  } else {
    document.documentElement.setAttribute('data-theme', 'dark');
  }
})();

function toggleTheme() {
  var current = document.documentElement.getAttribute('data-theme') || 'dark';
  var next = current === 'dark' ? 'light' : 'dark';
  document.documentElement.setAttribute('data-theme', next);
  localStorage.setItem('chub-theme', next);
}

// CATALOG is injected by the build as: const CATALOG=...;
var PER_PAGE = 60;
var query = '', activeLang = '', page = 0;

var $q = document.getElementById('q'), $grid = document.getElementById('grid'),
    $info = document.getElementById('info'), $filters = document.getElementById('filters'),
    $pg = document.getElementById('pagination'),
    $modal = document.getElementById('modal'), $modalTitle = document.getElementById('modal-title'),
    $modalBody = document.getElementById('modal-body'), $modalClose = document.getElementById('modal-close'),
    $modalLang = document.getElementById('modal-lang'), $modalVer = document.getElementById('modal-ver');

// Build language filter pills
var langs = [].concat.apply([], CATALOG.map(function(e) { return e.langNames; }));
langs = langs.filter(function(v, i, a) { return a.indexOf(v) === i; }).sort();
langs.forEach(function(l) {
  var b = document.createElement('span');
  b.className = 'pill'; b.textContent = l;
  b.onclick = function() {
    activeLang = activeLang === l ? '' : l; render();
    document.querySelectorAll('.pill').forEach(function(p) { p.classList.toggle('active', p.textContent === activeLang); });
  };
  $filters.appendChild(b);
});

function score(e, terms) {
  if (!terms.length) return 1;
  var s = 0, id = e.id.toLowerCase(), name = e.name.toLowerCase(),
      desc = e.description.toLowerCase(), tags = e.tags.join(' ').toLowerCase();
  for (var ti = 0; ti < terms.length; ti++) {
    var t = terms[ti];
    if (id.includes(t)) s += 10;
    if (name.includes(t)) s += 8;
    if (tags.includes(t)) s += 4;
    if (desc.includes(t)) s += 2;
  }
  return s;
}

function render() {
  var terms = query.toLowerCase().split(/\s+/).filter(function(t) { return t.length > 0; });
  var filtered = CATALOG.map(function(e) { return { e: e, s: score(e, terms) }; }).filter(function(x) { return x.s > 0; });
  if (terms.length) filtered.sort(function(a, b) { return b.s - a.s; });
  if (activeLang) filtered = filtered.filter(function(x) { return x.e.langNames.includes(activeLang); });
  var total = filtered.length;
  var maxPage = Math.max(0, Math.ceil(total / PER_PAGE) - 1);
  if (page > maxPage) page = maxPage;
  var slice = filtered.slice(page * PER_PAGE, (page + 1) * PER_PAGE);
  $info.textContent = total === CATALOG.length
    ? 'Showing ' + slice.length + ' of ' + total + ' entries'
    : total + ' result' + (total !== 1 ? 's' : '') + ' \u2014 showing ' + (page * PER_PAGE + 1) + '-' + Math.min((page + 1) * PER_PAGE, total);
  $grid.innerHTML = slice.map(function(x) { return card(x.e); }).join('');
  $pg.innerHTML = total > PER_PAGE
    ? '<button onclick="page=Math.max(0,page-1);render()" ' + (page === 0 ? 'disabled' : '') + '>Prev</button>' +
      '<span style="color:var(--muted);font-size:.85rem;padding:.4rem">Page ' + (page + 1) + ' / ' + (maxPage + 1) + '</span>' +
      '<button onclick="page=Math.min(' + maxPage + ',page+1);render()" ' + (page >= maxPage ? 'disabled' : '') + '>Next</button>'
    : '';
  // Attach click handlers
  document.querySelectorAll('.entry[data-id]').forEach(function(el) {
    el.onclick = function() { openDoc(el.dataset.id); };
  });
}

function card(e) {
  var badge = e.type === 'skill' ? '<span class="badge badge-skill">Skill</span>' : '<span class="badge badge-doc">Doc</span>';
  var src = e.source !== 'community' ? '<span class="badge badge-src">' + esc(e.source) + '</span>' : '';
  var langTags = e.langNames.map(function(l) { return '<span class="lang-tag">' + esc(l) + '</span>'; }).join('');
  var tags = e.tags.slice(0, 5).map(function(t) { return '<span class="tag">' + esc(t) + '</span>'; }).join('');
  return '<div class="entry" data-id="' + esc(e.id) + '" title="Click to view docs">' +
    '<div class="entry-head"><h3>' + esc(e.name) + '</h3>' + badge + src + '</div>' +
    '<div class="entry-id">' + esc(e.id) + '</div>' +
    '<p>' + esc(e.description) + '</p>' +
    '<div class="entry-meta">' + langTags + tags + '</div></div>';
}

function esc(s) { return s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;'); }

// --- Doc viewer ---
var currentEntry = null;

function openDoc(id) {
  currentEntry = CATALOG.find(function(e) { return e.id === id; });
  if (!currentEntry) return;
  $modalTitle.textContent = currentEntry.name;
  $modalBody.innerHTML = '<div class="loading">Loading...</div>';
  $modalBody.className = 'modal-body loading';
  $modal.classList.add('open');
  document.body.style.overflow = 'hidden';

  if (currentEntry.type === 'skill') {
    $modalLang.style.display = 'none';
    $modalVer.style.display = 'none';
    fetchAndRender(currentEntry.path + '/SKILL.md');
  } else if (currentEntry.langs.length > 0) {
    $modalLang.innerHTML = currentEntry.langs.map(function(l) { return '<option value="' + l.language + '">' + l.language + '</option>'; }).join('');
    $modalLang.style.display = 'inline-block';
    $modalLang.onchange = function() { updateVersions(); };
    updateVersions();
  } else {
    $modalBody.innerHTML = '<div class="loading">No content available.</div>';
  }
}

function updateVersions() {
  var lang = currentEntry.langs.find(function(l) { return l.language === $modalLang.value; });
  if (!lang) return;
  if (lang.versions.length > 1) {
    $modalVer.innerHTML = lang.versions.map(function(v) { return '<option value="' + v.path + '"' + (v.version === lang.recommended ? ' selected' : '') + '>' + v.version + '</option>'; }).join('');
    $modalVer.style.display = 'inline-block';
  } else {
    $modalVer.style.display = 'none';
  }
  var ver = lang.versions.find(function(v) { return v.version === lang.recommended; }) || lang.versions[0];
  if (ver) fetchAndRender(ver.path + '/DOC.md');
}

$modalVer.onchange = function() { fetchAndRender($modalVer.value + '/DOC.md'); };

function fetchAndRender(path) {
  $modalBody.innerHTML = '<div class="loading">Loading...</div>';
  $modalBody.className = 'modal-body loading';
  fetch('/' + path).then(function(r) {
    if (!r.ok) throw new Error(r.status);
    return r.text();
  }).then(function(text) {
    var stripped = text.replace(/^---[\s\S]*?---\s*/, '');
    $modalBody.innerHTML = '<div class="md">' + renderMd(stripped) + '</div>';
    $modalBody.className = 'modal-body';
  }).catch(function() {
    $modalBody.innerHTML = '<div class="loading">Failed to load document.</div>';
    $modalBody.className = 'modal-body loading';
  });
}

function closeModal() {
  $modal.classList.remove('open');
  document.body.style.overflow = '';
}
$modalClose.onclick = closeModal;
$modal.onclick = function(e) { if (e.target === $modal) closeModal(); };
document.addEventListener('keydown', function(e) { if (e.key === 'Escape') closeModal(); });

// Simple markdown-to-HTML renderer
function renderMd(src) {
  var html = '';
  var lines = src.split('\n');
  var i = 0, inCode = false, codeBuf = '';

  while (i < lines.length) {
    var line = lines[i];

    // Fenced code blocks
    if (!inCode && line.match(/^```/)) {
      inCode = true; codeBuf = ''; i++; continue;
    }
    if (inCode) {
      if (line.match(/^```/)) {
        html += '<pre><code>' + esc(codeBuf) + '</code></pre>';
        inCode = false; i++; continue;
      }
      codeBuf += line + '\n'; i++; continue;
    }

    // Headings
    var hm = line.match(/^(#{1,6})\s+(.*)/);
    if (hm) { html += '<h' + hm[1].length + '>' + inline(hm[2]) + '</h' + hm[1].length + '>'; i++; continue; }

    // Horizontal rule
    if (line.match(/^(-{3,}|\*{3,}|_{3,})\s*$/)) { html += '<hr>'; i++; continue; }

    // Blockquote
    if (line.match(/^>\s?/)) {
      var bq = '';
      while (i < lines.length && lines[i].match(/^>\s?/)) { bq += lines[i].replace(/^>\s?/, '') + '\n'; i++; }
      html += '<blockquote>' + renderMd(bq) + '</blockquote>'; continue;
    }

    // Unordered list
    if (line.match(/^\s*[-*+]\s/)) {
      html += '<ul>';
      while (i < lines.length && lines[i].match(/^\s*[-*+]\s/)) {
        html += '<li>' + inline(lines[i].replace(/^\s*[-*+]\s/, '')) + '</li>'; i++;
      }
      html += '</ul>'; continue;
    }

    // Ordered list
    if (line.match(/^\s*\d+\.\s/)) {
      html += '<ol>';
      while (i < lines.length && lines[i].match(/^\s*\d+\.\s/)) {
        html += '<li>' + inline(lines[i].replace(/^\s*\d+\.\s/, '')) + '</li>'; i++;
      }
      html += '</ol>'; continue;
    }

    // Table
    if (line.includes('|') && i + 1 < lines.length && lines[i + 1].match(/^\|?\s*[-:]+/)) {
      var hdrs = parseTRow(line);
      i += 2;
      html += '<table><thead><tr>' + hdrs.map(function(h) { return '<th>' + inline(h) + '</th>'; }).join('') + '</tr></thead><tbody>';
      while (i < lines.length && lines[i].includes('|')) {
        var cells = parseTRow(lines[i]);
        html += '<tr>' + cells.map(function(c) { return '<td>' + inline(c) + '</td>'; }).join('') + '</tr>'; i++;
      }
      html += '</tbody></table>'; continue;
    }

    // Empty line
    if (!line.trim()) { i++; continue; }

    // Paragraph
    var para = '';
    while (i < lines.length && lines[i].trim() && !lines[i].match(/^(#|```|>|\s*[-*+]\s|\s*\d+\.\s|---|\*\*\*|___|\|)/)) {
      para += (para ? ' ' : '') + lines[i]; i++;
    }
    if (para) html += '<p>' + inline(para) + '</p>';
  }
  if (inCode) html += '<pre><code>' + esc(codeBuf) + '</code></pre>';
  return html;
}

function inline(s) {
  return s
    .replace(/!\[([^\]]*)\]\(([^)]+)\)/g, '<img src="$2" alt="$1">')
    .replace(/\[([^\]]+)\]\(([^)]+)\)/g, '<a href="$2">$1</a>')
    .replace(/`([^`]+)`/g, '<code>$1</code>')
    .replace(/\*\*([^*]+)\*\*/g, '<strong>$1</strong>')
    .replace(/\*([^*]+)\*/g, '<em>$1</em>');
}

function parseTRow(line) {
  return line.replace(/^\|/, '').replace(/\|$/, '').split('|').map(function(c) { return c.trim(); });
}

var debounce;
$q.addEventListener('input', function() {
  clearTimeout(debounce);
  debounce = setTimeout(function() { query = $q.value; page = 0; render(); }, 120);
});
render();
