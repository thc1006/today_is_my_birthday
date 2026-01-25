/**
 * 共用前端模組 - 只需修改這個檔案即可套用到所有頁面
 *
 * 功能：
 * - Google Analytics 追蹤
 * - 訪客計數器
 * - 頁尾動態內容（年份、最後更新時間）
 */

export {};

// ============================================
// 類型定義
// ============================================

interface GtagFunction {
  (...args: unknown[]): void;
}

// ============================================
// 設定區 - 修改這裡的值即可全站生效
// ============================================

interface Config {
  readonly GA_ID: string;
  readonly VISITOR_BADGE_PAGE_ID: string;
}

const CONFIG: Config = {
  GA_ID: 'G-9WNYT414RM',
  VISITOR_BADGE_PAGE_ID: 'thc1006.us',
};

// ============================================
// Google Analytics
// ============================================

function initGoogleAnalytics(gaId: string): void {
  // 載入 gtag.js
  const script = document.createElement('script');
  script.async = true;
  script.src = `https://www.googletagmanager.com/gtag/js?id=${gaId}`;
  document.head.appendChild(script);

  // 初始化 dataLayer 和 gtag
  const win = window as typeof window & {
    dataLayer: unknown[];
    gtag: GtagFunction;
  };

  win.dataLayer = win.dataLayer || [];

  const gtag: GtagFunction = (...args: unknown[]) => {
    win.dataLayer.push(args);
  };

  win.gtag = gtag;
  gtag('js', new Date());
  gtag('config', gaId);
}

// ============================================
// 訪客計數器
// ============================================

function addVisitorBadge(pageId: string): void {
  const footer = document.querySelector('.site-footer');
  if (!footer || footer.querySelector('.visitor-badge')) {
    return;
  }

  const container = document.createElement('p');
  container.className = 'visitor-badge';

  const img = document.createElement('img');
  img.src = `https://visitor-badge.laobi.icu/badge?page_id=${pageId}`;
  img.alt = '訪客人數';
  img.loading = 'lazy';

  container.appendChild(img);
  footer.appendChild(container);
}

// ============================================
// 頁尾動態內容
// ============================================

function updateFooterContent(): void {
  // 更新年份
  const yearElement = document.getElementById('y');
  if (yearElement) {
    yearElement.textContent = new Date().getFullYear().toString();
  }

  // 更新最後修改時間
  const lastModifiedElement = document.getElementById('lm');
  if (lastModifiedElement) {
    lastModifiedElement.textContent = document.lastModified;
  }
}

// ============================================
// 初始化
// ============================================

function init(): void {
  // Google Analytics（立即初始化）
  initGoogleAnalytics(CONFIG.GA_ID);

  // DOM 載入完成後執行
  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', onDOMReady);
  } else {
    onDOMReady();
  }
}

function onDOMReady(): void {
  addVisitorBadge(CONFIG.VISITOR_BADGE_PAGE_ID);
  updateFooterContent();
}

// 執行初始化
init();
