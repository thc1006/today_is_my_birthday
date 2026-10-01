/**
 * 網站共用的 JavaScript：主題切換、捲動淡入、導覽列標示、頁尾、帳本頁的月份分組與搜尋。
 * pnpm run build 會把它編成 static/script.js，頁面載入的是編好的那份。
 *
 * 內容是 personal-homepage 的 script.js 改寫成 TypeScript，各段的編號沿用原檔，方便對照。
 * 只有一處刻意不同：訪客計數器用 thc1006.us 自己的計數，原檔用的是 NYCU 網站的計數。
 *
 * 上游改了 script.js 時，同步腳本會停下來。照上游的改動修改這個檔案、跑 pnpm run build，
 * 再把 scripts/homepage-upstream.lock 的 SCRIPT_SHA256 換成同步腳本印出的值。
 */

export {};

declare global {
  interface Window {
    dataLayer: unknown[];
    gtag: (...args: unknown[]) => void;
  }
}

/* ========================================================= */
/* 1. Theme toggle (light / dark, persisted in localStorage)  */
/* ========================================================= */
((): void => {
  const btn = document.getElementById('themeToggle');
  if (!btn) return;
  btn.addEventListener('click', () => {
    const cur = document.documentElement.dataset.theme || 'light';
    const next = cur === 'dark' ? 'light' : 'dark';
    document.documentElement.dataset.theme = next;
    localStorage.setItem('theme', next);
  });
})();

/* ========================================================= */
/* 2. Scroll-triggered reveal animations                      */
/* ========================================================= */
((): void => {
  if (!('IntersectionObserver' in window)) {
    document.querySelectorAll('[data-reveal]').forEach((el) => el.classList.add('in-view'));
    return;
  }
  const io = new IntersectionObserver(
    (entries) => {
      entries.forEach((e) => {
        if (e.isIntersecting) {
          e.target.classList.add('in-view');
          io.unobserve(e.target);
        }
      });
    },
    // threshold 0（不是 0.1）：0.1 是「目標自身高度的 10%」，高區塊（如 Foundation 完整清單 4861px）
    // 得先捲進 486px 才觸發，頂端露出後仍留一段空白死區。改 0 → 一進視窗就淡入，與區塊高度無關。
    // rootMargin 底部 -50px 確保是「有意義地進場」（露出約 50px）才觸發，而非 1px 微露就閃。
    { threshold: 0, rootMargin: '0px 0px -50px 0px' },
  );
  document.querySelectorAll('[data-reveal]').forEach((el) => io.observe(el));
})();

/* ========================================================= */
/* 3a. Active page indicator in topnav                        */
/* ========================================================= */
((): void => {
  // 本站在網域根目錄，第一個 replace 不會命中；保留是為了和上游同一套路徑規則。
  const path = location.pathname.replace(/^.*?\/~hctsai1006\//, '/').replace(/\/$/, '/');
  document.querySelectorAll<HTMLAnchorElement>('.topnav a[data-page]').forEach((a) => {
    const key = '/' + a.dataset.page + '/';
    if (path.startsWith(key)) {
      a.classList.add('active');
    }
  });
})();

/* ========================================================= */
/* 3. Topbar scrolled state                                   */
/* ========================================================= */
((): void => {
  const bar = document.getElementById('topbar');
  if (!bar) return;

  /* 以非同步交集通知取代 scroll handler 內的 window.scrollY 幾何讀取。
     這樣不會在樣式剛失效時強迫瀏覽器同步重算整個版面。 */
  if (!('IntersectionObserver' in window)) {
    bar.classList.add('scrolled');
    return;
  }

  const sentinel = document.createElement('span');
  sentinel.className = 'scroll-sentinel';
  sentinel.setAttribute('aria-hidden', 'true');
  bar.before(sentinel);

  const observer = new IntersectionObserver(([entry]) => {
    bar.classList.toggle('scrolled', !entry.isIntersecting);
  });
  observer.observe(sentinel);
})();

/* ========================================================= */
/* 4. Fetch latest 3 blog posts from /blog/index.xml          */
/* ========================================================= */
((): void => {
  const list = document.getElementById('blogList');
  if (!list) return;

  const xmlURL = 'blog/index.xml';

  fetch(xmlURL)
    .then((r) => {
      if (!r.ok) throw new Error('HTTP ' + r.status);
      return r.text();
    })
    .then((text) => {
      const xml = new DOMParser().parseFromString(text, 'application/xml');
      const items = Array.from(xml.querySelectorAll('item')).slice(0, 3);

      if (items.length === 0) {
        list.innerHTML = '<li class="blog-loading">Blog 還沒有文章。<a href="blog/">前往新增 →</a></li>';
        return;
      }

      list.innerHTML = '';
      items.forEach((item) => {
        const title = item.querySelector('title')?.textContent || '(無標題)';
        const link = item.querySelector('link')?.textContent || 'blog/';
        const pub = item.querySelector('pubDate')?.textContent;
        const date = pub ? new Date(pub) : null;
        const dateStr = date
          ? `${date.getFullYear()}-${String(date.getMonth() + 1).padStart(2, '0')}-${String(date.getDate()).padStart(2, '0')}`
          : '';

        const li = document.createElement('li');
        li.innerHTML = `
          <a href="${link}">${title}</a>
          <time datetime="${date ? date.toISOString() : ''}">${dateStr}</time>
        `;
        list.appendChild(li);
      });
    })
    .catch((err) => {
      list.innerHTML = `<li class="blog-loading">無法載入 Blog 摘要 (<code>${err.message}</code>)。<a href="blog/">直接前往 →</a></li>`;
    });
})();

/* ========================================================= */
/* 5. Live-site utilities                                     */
/*    集中在共用腳本，避免額外的關鍵請求。                    */
/* ========================================================= */
((): void => {
  const GA_ID = 'G-9WNYT414RM';
  const VISITOR_BADGE_PAGE_ID = 'thc1006.us';

  const loadAnalytics = (): void => {
    const script = document.createElement('script');
    script.async = true;
    script.src = `https://www.googletagmanager.com/gtag/js?id=${GA_ID}`;
    document.head.appendChild(script);

    window.dataLayer = window.dataLayer || [];
    window.gtag = (...args): number => window.dataLayer.push(args);
    window.gtag('js', new Date());
    window.gtag('config', GA_ID);
  };

  /* Analytics 不參與首屏呈現；等 load 後的空閒時段再載入。 */
  const scheduleAnalytics = (): void => {
    const scheduleIdle = (): void => {
      if ('requestIdleCallback' in window) {
        window.requestIdleCallback(loadAnalytics, { timeout: 4000 });
      } else {
        setTimeout(loadAnalytics, 0);
      }
    };
    window.setTimeout(scheduleIdle, 1500);
  };

  const updateFooter = (): void => {
    const year = document.getElementById('y');
    if (year) year.textContent = new Date().getFullYear().toString();

    const pageVersion = document.getElementById('pageVersion') as HTMLTimeElement | null;
    if (pageVersion) {
      const date = new Date(document.lastModified);
      if (!Number.isNaN(date.getTime())) {
        const isoDate = `${date.getFullYear()}-${String(date.getMonth() + 1).padStart(2, '0')}-${String(date.getDate()).padStart(2, '0')}`;
        pageVersion.textContent = isoDate;
        pageVersion.dateTime = isoDate;
      }
    }

    const footer = document.querySelector('.site-footer');
    if (!footer || footer.querySelector('.visitor-badge')) return;

    const badge = document.createElement('p');
    badge.className = 'visitor-badge';
    const image = document.createElement('img');
    image.src = `https://visitor-badge.laobi.icu/badge?page_id=${VISITOR_BADGE_PAGE_ID}`;
    image.alt = '訪客人數';
    image.loading = 'lazy';
    badge.appendChild(image);
    footer.appendChild(badge);
  };

  updateFooter();
  if (document.readyState === 'complete') scheduleAnalytics();
  else window.addEventListener('load', scheduleAnalytics, { once: true });
})();

/* ========================================================= */
/* 6. Ledger：merged PR 依月份瀏覽與篩選                      */
/* ========================================================= */
/* 靜態 HTML 維持一份完整的 <ol class="merge-list">，沒有 JS 的讀者照樣讀得到全部。
   這裡只在瀏覽器裡把它分成月份、每月先顯示 12 筆，其餘都在一次點擊內。
   2026-09-29 以前是 CSS-only 摺疊，以 max-height: 12000px 當「展開」；清單長到 44,599px
   （手機 89,908px）之後，按「顯示全部 381 筆」只看得到 55 筆（手機 26 筆），其餘被裁掉。 */
interface MonthBucket {
  key: string;
  items: HTMLLIElement[];
}

interface MonthGroup extends MonthBucket {
  expanded: boolean;
  det: HTMLDetailsElement;
  cnt: HTMLSpanElement;
  more?: HTMLButtonElement;
  printOpen?: boolean;
}

((): void => {
  const list = document.querySelector('#merged ol.merge-list');
  if (!list) return;
  const section = list.closest('section');
  if (!section) return;
  const items = [...list.children].filter((el): el is HTMLLIElement => el.tagName === 'LI');
  if (!items.length) return;

  const PER_MONTH = 12;
  const total = items.length;
  const tools = section.querySelector<HTMLElement>('.merge-tools');
  const input = section.querySelector<HTMLInputElement>('.merge-filter');
  const result = section.querySelector<HTMLElement>('.merge-result');

  const buckets: MonthBucket[] = [];
  const byKey = new Map<string, MonthBucket>();
  items.forEach((li) => {
    const date = (li.querySelector('.merge-date')?.textContent || '').trim();
    const key = /^\d{4}-\d{2}/.test(date) ? date.slice(0, 7) : '其他';
    let b = byKey.get(key);
    if (!b) {
      b = { key, items: [] };
      byKey.set(key, b);
      buckets.push(b);
    }
    b.items.push(li);
    li.dataset.search = (li.textContent ?? '').toLowerCase().replace(/\s+/g, ' ');
  });

  const monthLabel = (key: string): string => {
    const [y, m] = key.split('-');
    return m ? `${y} 年 ${Number(m)} 月` : key;
  };

  const wrap = document.createElement('div');
  wrap.className = 'merge-months';
  const groups: MonthGroup[] = buckets.map((b, gi) => {
    const det = document.createElement('details');
    det.className = 'merge-month';
    det.open = gi === 0;
    const sum = document.createElement('summary');
    const lab = document.createElement('span');
    lab.className = 'mm-label';
    lab.textContent = monthLabel(b.key);
    const cnt = document.createElement('span');
    cnt.className = 'mm-count';
    cnt.textContent = `${b.items.length} 筆`;
    sum.append(lab, cnt);
    const ol = document.createElement('ol');
    ol.className = 'merge-list';
    b.items.forEach((li) => ol.appendChild(li));
    det.append(sum, ol);
    const g: MonthGroup = { ...b, expanded: false, det, cnt };
    if (g.items.length > PER_MONTH) {
      const more = document.createElement('button');
      more.type = 'button';
      more.className = 'merge-more';
      more.textContent = `顯示本月其餘 ${g.items.length - PER_MONTH} 筆`;
      more.addEventListener('click', () => {
        g.expanded = true;
        apply();
        g.items[PER_MONTH]?.querySelector('a')?.focus();
      });
      det.appendChild(more);
      g.more = more;
    }
    wrap.appendChild(det);
    return g;
  });
  list.replaceWith(wrap);
  if (tools) tools.hidden = false;

  let query = '';
  let savedOpen: boolean[] | null = null;
  const apply = (): void => {
    const terms = query.toLowerCase().split(/\s+/).filter(Boolean);
    const filtering = terms.length > 0;
    if (filtering && !savedOpen) savedOpen = groups.map((g) => g.det.open);
    let found = 0;
    groups.forEach((g, gi) => {
      let matched = 0;
      g.items.forEach((li, i) => {
        const hit = !filtering || terms.every((t) => (li.dataset.search ?? '').includes(t));
        if (hit) matched++;
        li.hidden = !hit || (!filtering && !g.expanded && i >= PER_MONTH);
      });
      found += matched;
      g.det.hidden = filtering && matched === 0;
      if (filtering && matched) g.det.open = true;
      if (!filtering && savedOpen) g.det.open = savedOpen[gi];
      g.cnt.textContent = filtering ? `${matched} / ${g.items.length} 筆` : `${g.items.length} 筆`;
      if (g.more) g.more.hidden = filtering || g.expanded;
    });
    if (!filtering) savedOpen = null;
    if (result) {
      result.textContent = !filtering ? ''
        : found ? `找到 ${found} 筆（共 ${total} 筆）`
          : `沒有符合「${query.trim()}」的 merged PR`;
    }
  };

  if (input) {
    let timer = 0;
    input.addEventListener('input', () => {
      clearTimeout(timer);
      timer = window.setTimeout(() => { query = input.value; apply(); }, 120);
    });
    input.addEventListener('keydown', (e) => {
      if (e.key === 'Escape' && input.value) { input.value = ''; query = ''; apply(); }
    });
    const q = new URLSearchParams(location.search).get('q');
    if (q) { input.value = q; query = q; }
  }

  // 列印時把整份帳本攤開，印完再還原。
  window.addEventListener('beforeprint', () => {
    groups.forEach((g) => {
      g.printOpen = g.det.open;
      g.det.hidden = false;
      g.det.open = true;
      g.items.forEach((li) => { li.hidden = false; });
    });
  });
  window.addEventListener('afterprint', () => {
    groups.forEach((g) => { g.det.open = g.printOpen ?? false; });
    apply();
  });

  apply();
})();
