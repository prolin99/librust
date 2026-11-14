pub const HTML_PAGE: &str = r#"<!DOCTYPE html>
<html lang="zh-TW">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>現代化圖書管理借還系統</title>
  <style>
    :root {
      --primary: #2563eb;
      --primary-hover: #1d4ed8;
      --success: #16a34a;
      --danger: #dc2626;
      --warning: #d97706;
      --bg: #f8fafc;
      --card-bg: #ffffff;
      --text: #1e293b;
      --text-muted: #64748b;
      --border: #e2e8f0;
    }
    * { box-sizing: border-box; margin: 0; padding: 0; font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, "Noto Sans TC", sans-serif; }
    body { background-color: var(--bg); color: var(--text); display: flex; flex-direction: column; min-height: 100vh; }
    
    /* 頂部導航 */
    header { background: #1e293b; color: white; padding: 0.75rem 1.5rem; display: flex; justify-content: space-between; align-items: center; box-shadow: 0 2px 4px rgba(0,0,0,0.1); }
    .brand { display: flex; align-items: center; gap: 0.75rem; font-size: 1.25rem; font-weight: bold; }
    .brand-icon { font-size: 1.5rem; }
    nav { display: flex; gap: 0.5rem; flex-wrap: wrap; }
    nav button { background: transparent; border: none; color: #94a3b8; padding: 0.5rem 0.85rem; border-radius: 0.375rem; cursor: pointer; font-size: 0.95rem; font-weight: 500; transition: all 0.2s; display: flex; align-items: center; gap: 0.4rem; }
    nav button:hover { color: white; background: rgba(255,255,255,0.08); }
    nav button.active { color: white; background: var(--primary); }

    /* 主區域 */
    main { flex: 1; padding: 1.5rem; max-width: 1400px; margin: 0 auto; width: 100%; }
    .tab-content { display: none; }
    .tab-content.active { display: block; }

    /* 卡片與格線 */
    .card { background: var(--card-bg); border-radius: 0.5rem; border: 1px solid var(--border); box-shadow: 0 1px 3px rgba(0,0,0,0.05); padding: 1.25rem; margin-bottom: 1.5rem; }
    .grid { display: grid; gap: 1rem; }
    .grid-2 { grid-template-columns: repeat(auto-fit, minmax(320px, 1fr)); }
    .grid-4 { grid-template-columns: repeat(auto-fit, minmax(220px, 1fr)); }

    /* 按鈕與輸入框 */
    .btn { display: inline-flex; align-items: center; justify-content: center; gap: 0.35rem; padding: 0.5rem 1rem; font-size: 0.9rem; font-weight: 500; border-radius: 0.375rem; border: none; cursor: pointer; transition: background 0.15s; }
    .btn-primary { background: var(--primary); color: white; }
    .btn-primary:hover { background: var(--primary-hover); }
    .btn-success { background: var(--success); color: white; }
    .btn-danger { background: var(--danger); color: white; }
    .btn-warning { background: var(--warning); color: white; }
    .btn-outline { background: transparent; border: 1px solid var(--border); color: var(--text); }
    .btn-outline:hover { background: #f1f5f9; }
    .btn-sm { padding: 0.25rem 0.5rem; font-size: 0.8rem; }
    
    input, select, textarea { width: 100%; padding: 0.5rem 0.75rem; border: 1px solid var(--border); border-radius: 0.375rem; font-size: 0.9rem; color: var(--text); background: white; }
    input:focus, select:focus, textarea:focus { outline: 2px solid var(--primary); border-color: transparent; }

    /* 表格 */
    .table-container { overflow-x: auto; background: white; border-radius: 0.375rem; border: 1px solid var(--border); }
    table { width: 100%; border-collapse: collapse; text-align: left; font-size: 0.9rem; }
    th { background: #f8fafc; padding: 0.75rem 1rem; font-weight: 600; color: var(--text-muted); border-bottom: 1px solid var(--border); }
    td { padding: 0.75rem 1rem; border-bottom: 1px solid var(--border); vertical-align: middle; }
    tr:hover td { background: #f8fafc; }

    /* 狀態徽章 */
    .badge { display: inline-block; padding: 0.2rem 0.5rem; font-size: 0.75rem; font-weight: 600; border-radius: 9999px; }
    .badge-success { background: #dcfce7; color: #15803d; }
    .badge-danger { background: #fee2e2; color: #b91c1c; }
    .badge-warning { background: #fef3c7; color: #b45309; }
    .badge-info { background: #e0f2fe; color: #0369a1; }

    /* 書籍卡片 */
    .book-cover { width: 48px; height: 68px; object-fit: cover; border-radius: 0.25rem; border: 1px solid var(--border); }
    .cover-fallback { width: 48px; height: 68px; border-radius: 0.25rem; display: flex; flex-direction: column; justify-content: center; align-items: center; color: white; padding: 0.2rem; text-align: center; font-size: 0.65rem; font-weight: bold; overflow: hidden; }

    /* 借還書作業區專用樣式 */
    .circulation-box { background: #f8fafc; border: 2px dashed #cbd5e1; border-radius: 0.5rem; padding: 1.5rem; }
    .scan-input { font-size: 1.25rem; padding: 0.75rem 1rem; font-weight: bold; letter-spacing: 0.05em; }

    /* Modal 彈窗 */
    .modal-overlay { position: fixed; inset: 0; background: rgba(0,0,0,0.5); display: none; align-items: center; justify-content: center; z-index: 50; padding: 1rem; }
    .modal-overlay.active { display: flex; }
    .modal { background: white; border-radius: 0.5rem; max-width: 600px; width: 100%; max-height: 90vh; overflow-y: auto; padding: 1.5rem; box-shadow: 0 10px 25px rgba(0,0,0,0.2); }
    .modal-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 1rem; padding-bottom: 0.5rem; border-bottom: 1px solid var(--border); }

    /* 列印專區樣式 */
    @media print {
      @page { margin: 5mm; size: auto; }
      body { background: white !important; margin: 0; padding: 0; }
      body * { visibility: hidden; }
      #print-area, #print-area * { visibility: visible; }
      #print-area { position: absolute; left: 0; top: 0; width: 100%; padding: 0 !important; margin: 0 !important; border: none !important; }
      header, nav, .btn, .no-print, footer { display: none !important; }
      .print-tag-item { page-break-inside: avoid; break-inside: avoid; }
    }
  </style>
</head>
<body>

  <header>
    <div class="brand">
      <span class="brand-icon">📚</span>
      <span id="header-library-name">圖書管理借還系統</span>
    </div>
    <nav>
      <button class="active" onclick="switchTab('circulation')">🔄 借還櫃台</button>
      <button onclick="switchTab('active-loans')">📋 在借與催還</button>
      <button onclick="switchTab('books')">📖 書籍管理</button>
      <button onclick="switchTab('readers')">👥 讀者名冊</button>
      <button onclick="switchTab('roles')">🏷️ 身分別設定</button>
      <button onclick="switchTab('enrichment')">🌐 網路補齊</button>
      <button onclick="switchTab('print')">🖨️ 證卡標籤列印</button>
      <button onclick="switchTab('stats')">📊 統計儀表板</button>
      <button onclick="switchTab('settings')">⚙️ 系統設定</button>
    </nav>
  </header>

  <main>
    <!-- 1. 借還櫃台 -->
    <section id="tab-circulation" class="tab-content active">
      <div class="grid grid-2">
        <!-- 借書作業 -->
        <div class="card">
          <h2 style="margin-bottom: 1rem; color: var(--primary);">📥 讀者借書作業</h2>
          <div style="margin-bottom: 1rem;">
            <label style="font-weight: 600; display: block; margin-bottom: 0.25rem;">1. 刷入讀者證號 (或輸入後按 Enter)</label>
            <input type="text" id="borrow-reader-id" class="scan-input" placeholder="請刷入讀者條碼/證號..."
                   oninput="handleReaderIdInput(this.value)"
                   onchange="handleReaderIdChange(this.value)"
                   onkeydown="if(event.key==='Enter') { handleReaderIdChange(this.value); }">
            <div id="borrow-reader-error" style="display: none; color: var(--danger); font-size: 0.85rem; margin-top: 0.35rem; font-weight: 600;"></div>
          </div>
          <!-- 讀者基本資訊 -->
          <div id="borrow-reader-info" style="display: none; background: #eff6ff; border: 1px solid #bfdbfe; border-radius: 0.375rem; padding: 0.75rem; margin-bottom: 0.75rem;">
            <div style="display: flex; justify-content: space-between; align-items: center;">
              <strong id="bri-name" style="font-size: 1.1rem;">-</strong>
              <span id="bri-role" class="badge badge-info">-</span>
            </div>
            <div style="margin-top: 0.5rem; font-size: 0.85rem; color: #1e40af;">
              目前已借：<span id="bri-active">0</span> 本 / 可借上限：<span id="bri-max">0</span> 本 | 可借天數：<span id="bri-days">0</span> 天
            </div>
          </div>

          <!-- 逾期警告橫幅（如有逾期時顯示） -->
          <div id="borrow-reader-overdue-alert" style="display: none; background: #fef2f2; border: 1px solid #f87171; border-radius: 0.375rem; padding: 0.75rem; margin-bottom: 0.75rem; color: #991b1b; font-size: 0.88rem;">
            <div style="display: flex; align-items: center; gap: 0.5rem; font-weight: bold;">
              <span style="font-size: 1.25rem;">⚠️</span>
              <span id="bri-overdue-msg">該讀者尚有逾期圖書未還！依規定暫停借閱權限，在還清逾期書籍前無法借新書。</span>
            </div>
          </div>

          <!-- 讀者目前已借圖書清單 -->
          <div id="borrow-reader-loans-panel" style="display: none; margin-bottom: 1rem; border: 1px solid var(--border); border-radius: 0.375rem; overflow: hidden; background: #ffffff;">
            <div style="background: #f8fafc; padding: 0.5rem 0.75rem; font-size: 0.85rem; font-weight: 600; color: #334155; display: flex; justify-content: space-between; align-items: center; border-bottom: 1px solid var(--border);">
              <span>📚 該讀者目前在借圖書清單</span>
              <span id="bri-loans-count-badge" class="badge badge-secondary">0 本</span>
            </div>
            <div style="max-height: 220px; overflow-y: auto;">
              <table style="width: 100%; border-collapse: collapse; font-size: 0.82rem;" id="borrow-reader-loans-table">
                <thead>
                  <tr style="background: #f1f5f9; text-align: left; border-bottom: 1px solid var(--border);">
                    <th style="padding: 0.4rem 0.6rem;">條碼</th>
                    <th style="padding: 0.4rem 0.6rem;">書名</th>
                    <th style="padding: 0.4rem 0.6rem;">應還日期</th>
                    <th style="padding: 0.4rem 0.6rem;">狀態</th>
                    <th style="padding: 0.4rem 0.6rem; text-align: center;">操作</th>
                  </tr>
                </thead>
                <tbody></tbody>
              </table>
            </div>
          </div>
          <div style="margin-bottom: 1rem;">
            <label style="font-weight: 600; display: block; margin-bottom: 0.25rem;">2. 刷入書籍條碼 (按 Enter 自動完成借閱)</label>
            <input type="text" id="borrow-book-id" class="scan-input" placeholder="⚠️ 請先刷入有效讀者證號..." disabled
                   style="background: #f1f5f9; cursor: not-allowed;"
                   onkeydown="if(event.key==='Enter') executeBorrow()">
          </div>
          <button id="btn-submit-borrow" class="btn btn-primary" style="width: 100%; padding: 0.75rem;" onclick="executeBorrow()" disabled>確認完成借出</button>
        </div>

        <!-- 還書作業 -->
        <div class="card">
          <h2 style="margin-bottom: 1rem; color: var(--success);">📤 書籍歸還作業</h2>
          <div class="circulation-box" style="margin-bottom: 1rem; text-align: center;">
            <div style="font-size: 2.5rem; margin-bottom: 0.5rem;">📦</div>
            <p style="color: var(--text-muted); margin-bottom: 1rem;">將條碼掃描槍對準書籍條碼，刷入即可直接自動歸還！</p>
            <input type="text" id="return-book-id" class="scan-input" placeholder="請直接刷入書籍條碼..." onkeydown="if(event.key==='Enter') executeReturn()">
          </div>
          <button class="btn btn-success" style="width: 100%; padding: 0.75rem;" onclick="executeReturn()">確認歸還入庫</button>
          
          <div id="return-result-box" style="margin-top: 1rem; display: none; padding: 0.75rem; border-radius: 0.375rem;"></div>
        </div>
      </div>
    </section>

    <!-- 2. 在借與催還清單 -->
    <section id="tab-active-loans" class="tab-content">
      <div class="card">
        <div style="display: flex; justify-content: space-between; align-items: center; flex-wrap: wrap; gap: 0.5rem; margin-bottom: 0.75rem;">
          <div style="display: flex; gap: 0.5rem; flex: 1; max-width: 550px;">
            <input type="text" id="active-loans-search" placeholder="搜尋讀者姓名、證號、書名、書籍條碼..." onkeydown="if(event.key==='Enter') filterActiveLoans()">
            <select id="active-loans-status-filter" style="width: 170px;" onchange="filterActiveLoans()">
              <option value="">全部出借中</option>
              <option value="overdue">⚠️ 僅看逾期 (催還名單)</option>
              <option value="normal">✅ 正常借閱中 (未逾期)</option>
            </select>
            <button class="btn btn-primary" onclick="filterActiveLoans()">搜尋</button>
          </div>
          <div style="display: flex; gap: 0.5rem;">
            <button class="btn btn-outline" onclick="loadActiveLoans()">🔄 重新整理</button>
          </div>
        </div>

        <!-- 在借清單分頁控制列 (頂部) -->
        <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 0.75rem; background: #f8fafc; padding: 0.5rem 0.75rem; border-radius: 0.375rem; border: 1px solid var(--border); font-size: 0.85rem; flex-wrap: wrap; gap: 0.5rem;">
          <div style="display: flex; align-items: center; gap: 0.5rem;">
            <span>每頁顯示：</span>
            <select id="active-loans-page-size" style="width: 90px; padding: 0.25rem 0.5rem; font-size: 0.85rem;" onchange="changeActiveLoansPageSize(this.value)">
              <option value="10">10 筆</option>
              <option value="20" selected>20 筆</option>
              <option value="50">50 筆</option>
              <option value="100">100 筆</option>
              <option value="0">全部顯示</option>
            </select>
            <span id="active-loans-page-summary" style="color: var(--text-muted); margin-left: 0.5rem;">載入中...</span>
          </div>
          <div style="display: flex; align-items: center; gap: 0.35rem;">
            <button class="btn btn-outline btn-sm" id="btn-loans-first" onclick="gotoActiveLoansPage(1)">⏮ 首頁</button>
            <button class="btn btn-outline btn-sm" id="btn-loans-prev" onclick="prevActiveLoansPage()">◀ 上一頁</button>
            <span id="active-loans-page-indicator" style="font-weight: 600; padding: 0 0.5rem;">第 1 / 1 頁</span>
            <button class="btn btn-outline btn-sm" id="btn-loans-next" onclick="nextActiveLoansPage()">下一頁 ▶</button>
            <button class="btn btn-outline btn-sm" id="btn-loans-last" onclick="gotoActiveLoansLastPage()">末頁 ⏭</button>
          </div>
        </div>

        <div class="table-container">
          <table id="active-loans-table">
            <thead>
              <tr>
                <th>書籍條碼</th>
                <th>書名</th>
                <th>讀者證號</th>
                <th>借閱人姓名</th>
                <th>身分別</th>
                <th>借閱日期</th>
                <th>應還日期</th>
                <th>狀態/剩餘天數</th>
                <th>操作</th>
              </tr>
            </thead>
            <tbody></tbody>
          </table>
        </div>

        <!-- 在借清單分頁控制列 (底部) -->
        <div style="display: flex; justify-content: space-between; align-items: center; margin-top: 0.75rem; font-size: 0.85rem; flex-wrap: wrap; gap: 0.5rem;">
          <span id="active-loans-page-summary-bottom" style="color: var(--text-muted);"></span>
          <div style="display: flex; align-items: center; gap: 0.35rem;">
            <button class="btn btn-outline btn-sm" onclick="gotoActiveLoansPage(1)">⏮ 首頁</button>
            <button class="btn btn-outline btn-sm" onclick="prevActiveLoansPage()">◀ 上一頁</button>
            <span id="active-loans-page-indicator-bottom" style="font-weight: 600; padding: 0 0.5rem;">第 1 / 1 頁</span>
            <button class="btn btn-outline btn-sm" onclick="nextActiveLoansPage()">下一頁 ▶</button>
            <button class="btn btn-outline btn-sm" onclick="gotoActiveLoansLastPage()">末頁 ⏭</button>
          </div>
        </div>
      </div>
    </section>

    <!-- 3. 書籍管理 -->
    <section id="tab-books" class="tab-content">
      <div class="card">
        <div style="display: flex; justify-content: space-between; align-items: center; flex-wrap: wrap; gap: 0.5rem; margin-bottom: 0.75rem;">
          <div style="display: flex; gap: 0.5rem; flex: 1; max-width: 500px;">
            <input type="text" id="books-search" placeholder="搜尋書名、條碼、作者、ISBN、分類..." onkeydown="if(event.key==='Enter') loadBooks()">
            <select id="books-status-filter" style="width: 130px;" onchange="loadBooks()">
              <option value="">全部狀態</option>
              <option value="available">在館可借</option>
              <option value="borrowed">借出中</option>
            </select>
            <button class="btn btn-primary" onclick="loadBooks()">搜尋</button>
          </div>
          <div style="display: flex; gap: 0.5rem;">
            <button class="btn btn-outline" onclick="document.getElementById('import-books-input').click()">📥 匯入 Excel/CSV</button>
            <input type="file" id="import-books-input" style="display: none;" accept=".xlsx,.xls,.csv" onchange="uploadBooksFile(this)">
            <a href="/api/templates/books.csv" class="btn btn-outline" download>📄 下載範本</a>
            <button class="btn btn-primary" onclick="openBookModal()">➕ 新增書籍</button>
          </div>
        </div>

        <!-- 書籍分頁控制列 (頂部) -->
        <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 0.75rem; background: #f8fafc; padding: 0.5rem 0.75rem; border-radius: 0.375rem; border: 1px solid var(--border); font-size: 0.85rem; flex-wrap: wrap; gap: 0.5rem;">
          <div style="display: flex; align-items: center; gap: 0.5rem;">
            <span>每頁顯示：</span>
            <select id="books-page-size" style="width: 90px; padding: 0.25rem 0.5rem; font-size: 0.85rem;" onchange="changeBooksPageSize(this.value)">
              <option value="10">10 筆</option>
              <option value="20" selected>20 筆</option>
              <option value="50">50 筆</option>
              <option value="100">100 筆</option>
              <option value="0">全部顯示</option>
            </select>
            <span id="books-page-summary" style="color: var(--text-muted); margin-left: 0.5rem;">載入中...</span>
          </div>
          <div style="display: flex; align-items: center; gap: 0.35rem;">
            <button class="btn btn-outline btn-sm" id="btn-books-first" onclick="gotoBooksPage(1)">⏮ 首頁</button>
            <button class="btn btn-outline btn-sm" id="btn-books-prev" onclick="prevBooksPage()">◀ 上一頁</button>
            <span id="books-page-indicator" style="font-weight: 600; padding: 0 0.5rem;">第 1 / 1 頁</span>
            <button class="btn btn-outline btn-sm" id="btn-books-next" onclick="nextBooksPage()">下一頁 ▶</button>
            <button class="btn btn-outline btn-sm" id="btn-books-last" onclick="gotoBooksLastPage()">末頁 ⏭</button>
          </div>
        </div>

        <div class="table-container">
          <table id="books-table">
            <thead>
              <tr>
                <th style="width: 60px;">封面</th>
                <th>條碼/登錄號</th>
                <th>書名</th>
                <th>作者</th>
                <th>出版社</th>
                <th>ISBN</th>
                <th>分類</th>
                <th>存放地</th>
                <th>狀態</th>
                <th>借閱次數</th>
                <th>操作</th>
              </tr>
            </thead>
            <tbody></tbody>
          </table>
        </div>

        <!-- 書籍分頁控制列 (底部) -->
        <div style="display: flex; justify-content: space-between; align-items: center; margin-top: 0.75rem; font-size: 0.85rem; flex-wrap: wrap; gap: 0.5rem;">
          <span id="books-page-summary-bottom" style="color: var(--text-muted);"></span>
          <div style="display: flex; align-items: center; gap: 0.35rem;">
            <button class="btn btn-outline btn-sm" onclick="gotoBooksPage(1)">⏮ 首頁</button>
            <button class="btn btn-outline btn-sm" onclick="prevBooksPage()">◀ 上一頁</button>
            <span id="books-page-indicator-bottom" style="font-weight: 600; padding: 0 0.5rem;">第 1 / 1 頁</span>
            <button class="btn btn-outline btn-sm" onclick="nextBooksPage()">下一頁 ▶</button>
            <button class="btn btn-outline btn-sm" onclick="gotoBooksLastPage()">末頁 ⏭</button>
          </div>
        </div>
      </div>
    </section>

    <!-- 4. 讀者管理 -->
    <section id="tab-readers" class="tab-content">
      <div class="card">
        <div style="display: flex; justify-content: space-between; align-items: center; flex-wrap: wrap; gap: 0.5rem; margin-bottom: 0.75rem;">
          <div style="display: flex; gap: 0.5rem; flex: 1; max-width: 400px;">
            <input type="text" id="readers-search" placeholder="搜尋讀者姓名、證號、班級..." onkeydown="if(event.key==='Enter') loadReaders()">
            <button class="btn btn-primary" onclick="loadReaders()">搜尋</button>
          </div>
          <div style="display: flex; gap: 0.5rem;">
            <button class="btn btn-outline" onclick="document.getElementById('import-readers-input').click()">📥 匯入 Excel/CSV</button>
            <input type="file" id="import-readers-input" style="display: none;" accept=".xlsx,.xls,.csv" onchange="uploadReadersFile(this)">
            <a href="/api/templates/readers.csv" class="btn btn-outline" download>📄 下載範本</a>
            <button class="btn btn-primary" onclick="openReaderModal()">➕ 新增讀者</button>
          </div>
        </div>

        <!-- 讀者分頁控制列 (頂部) -->
        <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 0.75rem; background: #f8fafc; padding: 0.5rem 0.75rem; border-radius: 0.375rem; border: 1px solid var(--border); font-size: 0.85rem; flex-wrap: wrap; gap: 0.5rem;">
          <div style="display: flex; align-items: center; gap: 0.5rem;">
            <span>每頁顯示：</span>
            <select id="readers-page-size" style="width: 90px; padding: 0.25rem 0.5rem; font-size: 0.85rem;" onchange="changeReadersPageSize(this.value)">
              <option value="10">10 筆</option>
              <option value="20" selected>20 筆</option>
              <option value="50">50 筆</option>
              <option value="100">100 筆</option>
              <option value="0">全部顯示</option>
            </select>
            <span id="readers-page-summary" style="color: var(--text-muted); margin-left: 0.5rem;">載入中...</span>
          </div>
          <div style="display: flex; align-items: center; gap: 0.35rem;">
            <button class="btn btn-outline btn-sm" id="btn-readers-first" onclick="gotoReadersPage(1)">⏮ 首頁</button>
            <button class="btn btn-outline btn-sm" id="btn-readers-prev" onclick="prevReadersPage()">◀ 上一頁</button>
            <span id="readers-page-indicator" style="font-weight: 600; padding: 0 0.5rem;">第 1 / 1 頁</span>
            <button class="btn btn-outline btn-sm" id="btn-readers-next" onclick="nextReadersPage()">下一頁 ▶</button>
            <button class="btn btn-outline btn-sm" id="btn-readers-last" onclick="gotoReadersLastPage()">末頁 ⏭</button>
          </div>
        </div>

        <div class="table-container">
          <table id="readers-table">
            <thead>
              <tr>
                <th>證號</th>
                <th>姓名</th>
                <th>身分別</th>
                <th>可借上限</th>
                <th>可借天數</th>
                <th>班級/單位</th>
                <th>目前借閱</th>
                <th>狀態</th>
                <th>操作</th>
              </tr>
            </thead>
            <tbody></tbody>
          </table>
        </div>

        <!-- 讀者分頁控制列 (底部) -->
        <div style="display: flex; justify-content: space-between; align-items: center; margin-top: 0.75rem; font-size: 0.85rem; flex-wrap: wrap; gap: 0.5rem;">
          <span id="readers-page-summary-bottom" style="color: var(--text-muted);"></span>
          <div style="display: flex; align-items: center; gap: 0.35rem;">
            <button class="btn btn-outline btn-sm" onclick="gotoReadersPage(1)">⏮ 首頁</button>
            <button class="btn btn-outline btn-sm" onclick="prevReadersPage()">◀ 上一頁</button>
            <span id="readers-page-indicator-bottom" style="font-weight: 600; padding: 0 0.5rem;">第 1 / 1 頁</span>
            <button class="btn btn-outline btn-sm" onclick="nextReadersPage()">下一頁 ▶</button>
            <button class="btn btn-outline btn-sm" onclick="gotoReadersLastPage()">末頁 ⏭</button>
          </div>
        </div>
      </div>
    </section>

    <!-- 5. 身分別設定 -->
    <section id="tab-roles" class="tab-content">
      <div class="card">
        <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 1rem;">
          <div>
            <h2>🏷️ 讀者身分別與借閱規則設定</h2>
            <p style="color: var(--text-muted); font-size: 0.85rem; margin-top: 0.25rem;">可依身分（如服務對象、工作人員、教師、學生等）自訂各身分的最大借閱冊數與天數。</p>
          </div>
          <button class="btn btn-primary" onclick="openRoleModal()">➕ 新增身分別</button>
        </div>
        <div class="table-container">
          <table id="roles-table">
            <thead>
              <tr>
                <th>身分代碼</th>
                <th>身分名稱</th>
                <th>可借書籍冊數 (上限)</th>
                <th>可借天數 (天)</th>
                <th>續借天數 (天)</th>
                <th>備註說明</th>
                <th>操作</th>
              </tr>
            </thead>
            <tbody></tbody>
          </table>
        </div>
      </div>
    </section>

    <!-- 6. 網路書目補齊與防攻擊限速 -->
    <section id="tab-enrichment" class="tab-content">
      <div class="card">
        <h2>🌐 網路書目與封面圖自動補齊精靈 (具防攻擊限速)</h2>
        <p style="color: var(--text-muted); margin: 0.5rem 0 1.5rem 0;">系統會依設定之頻率在背景向 Google Books / Open Library 查詢書籍封面與簡介，絕不觸發網路阻斷防護。</p>
        
        <div class="grid grid-2" style="margin-bottom: 1.5rem;">
          <div style="background: #f8fafc; padding: 1rem; border-radius: 0.375rem; border: 1px solid var(--border);">
            <h4 style="margin-bottom: 0.75rem;">⚙️ 限速與執行策略</h4>
            <div style="margin-bottom: 0.5rem;">
              <label style="font-size: 0.85rem; font-weight: 600;">每分鐘連線頻率上限 (次/分)</label>
              <input type="number" id="enrich-rpm" value="20" min="5" max="60" style="margin-top: 0.25rem;">
              <small style="color: var(--text-muted);">建議設為 15~25 次/分，避免被判定為惡意連線。</small>
            </div>
            <div style="margin-top: 0.75rem;">
              <label><input type="checkbox" id="enrich-cover" checked> 自動下載並儲存封面圖片</label><br>
              <label><input type="checkbox" id="enrich-desc" checked> 補齊書籍內容簡介 (Description)</label>
            </div>
            <div style="margin-top: 1rem; display: flex; gap: 0.5rem;">
              <button class="btn btn-primary" onclick="startEnrichment()">▶️ 啟動背景批次補齊</button>
              <button class="btn btn-danger" onclick="stopEnrichment()">⏹️ 停止/暫停</button>
            </div>
          </div>

          <div style="background: #f8fafc; padding: 1rem; border-radius: 0.375rem; border: 1px solid var(--border);">
            <h4 style="margin-bottom: 0.75rem;">📊 目前補齊進度</h4>
            <div style="background: #e2e8f0; height: 1.25rem; border-radius: 9999px; overflow: hidden; margin-bottom: 0.5rem;">
              <div id="enrich-progress-bar" style="background: var(--primary); width: 0%; height: 100%; transition: width 0.3s;"></div>
            </div>
            <div style="display: flex; justify-content: space-between; font-size: 0.85rem; margin-bottom: 0.5rem;">
              <span id="enrich-progress-percent">0%</span>
              <span id="enrich-progress-count">0 / 0</span>
            </div>
            <div id="enrich-status-msg" style="font-size: 0.85rem; color: var(--text-muted); background: white; padding: 0.5rem; border-radius: 0.25rem; border: 1px solid var(--border); min-height: 40px;">待命狀態</div>
          </div>
        </div>
      </div>
    </section>

    <!-- 7. 證卡標籤批次列印 -->
    <section id="tab-print" class="tab-content">
      <div class="card">
        <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 1rem; flex-wrap: wrap; gap: 0.5rem;">
          <h2>🖨️ 借書證與書籍標籤批次列印</h2>
          <div style="display: flex; gap: 0.5rem;">
            <button class="btn btn-outline" onclick="previewBatchCards()">🪪 預覽借書證</button>
            <button class="btn btn-outline" onclick="previewBatchBarcodes()">🏷️ 預覽書籍標籤</button>
            <button class="btn btn-primary" onclick="window.print()">🖨️ 直接列印</button>
          </div>
        </div>

        <!-- 列印範圍自訂設定列 -->
        <div style="background: #f8fafc; border: 1px solid var(--border); border-radius: 0.375rem; padding: 1rem; margin-bottom: 1.25rem;">
          <div style="font-weight: 600; font-size: 0.9rem; margin-bottom: 0.5rem; color: #334155;">🎯 列印範圍設定（可指定起訖條碼/證號，或留空列印全部）：</div>
          <div style="display: grid; grid-template-columns: repeat(auto-fit, minmax(180px, 1fr)); gap: 0.75rem; align-items: flex-end;">
            <div>
              <label style="font-size: 0.8rem; color: var(--text-muted);">起始編號 (條碼/證號)：</label>
              <input type="text" id="print-range-start" placeholder="例：B000001 或 00001" style="margin-top: 0.25rem;">
            </div>
            <div>
              <label style="font-size: 0.8rem; color: var(--text-muted);">結束編號 (條碼/證號)：</label>
              <input type="text" id="print-range-end" placeholder="例：B000050 或 00050" style="margin-top: 0.25rem;">
            </div>
            <div>
              <label style="font-size: 0.8rem; color: var(--text-muted);">關鍵字 / 分類 / 身分篩選：</label>
              <input type="text" id="print-range-filter" placeholder="如書名、作者、班級..." style="margin-top: 0.25rem;">
            </div>
            <div>
              <label style="font-size: 0.8rem; color: var(--text-muted);">最多筆數限制：</label>
              <select id="print-range-limit" style="margin-top: 0.25rem;">
                <option value="0">全部符合項目</option>
                <option value="20">前 20 筆</option>
                <option value="50" selected>前 50 筆</option>
                <option value="100">前 100 筆</option>
                <option value="200">前 200 筆</option>
              </select>
            </div>
            <div style="display: flex; align-items: center; padding-bottom: 0.5rem;">
              <label style="font-size: 0.85rem; font-weight: 600; cursor: pointer; display: flex; align-items: center; gap: 0.35rem; color: #1e293b;">
                <input type="checkbox" id="print-opt-spine" checked style="width: 1rem; height: 1rem; cursor: pointer;"> 書籍標籤包含背標
              </label>
            </div>
          </div>
          <div id="print-status-bar" style="margin-top: 0.75rem; font-size: 0.85rem; color: #475569; display: flex; justify-content: space-between;">
            <span>💡 提示：設定起訖編號後，點選上方「預覽借書證」或「預覽書籍標籤」即可套用篩選。</span>
            <span id="print-count-label" style="font-weight: 600;"></span>
          </div>
        </div>

        <div id="print-area" style="background: white; padding: 1.5rem; border: 1px solid var(--border); border-radius: 0.375rem; min-height: 400px;">
          <p style="color: var(--text-muted); text-align: center; margin-top: 5rem;">請點選上方按鈕預覽借書證或書籍條碼標籤...</p>
        </div>
      </div>
    </section>

    <!-- 8. 統計儀表板 -->
    <section id="tab-stats" class="tab-content">
      <div class="grid grid-4" style="margin-bottom: 1.5rem;">
        <div class="card" style="text-align: center;">
          <div style="color: var(--text-muted); font-size: 0.85rem;">📚 總藏書量</div>
          <div id="stat-total-books" style="font-size: 2rem; font-weight: bold; margin-top: 0.25rem;">-</div>
        </div>
        <div class="card" style="text-align: center;">
          <div style="color: var(--text-muted); font-size: 0.85rem;">✅ 在館可借</div>
          <div id="stat-available-books" style="font-size: 2rem; font-weight: bold; color: var(--success); margin-top: 0.25rem;">-</div>
        </div>
        <div class="card" style="text-align: center;">
          <div style="color: var(--text-muted); font-size: 0.85rem;">📖 目前借出</div>
          <div id="stat-borrowed-books" style="font-size: 2rem; font-weight: bold; color: var(--primary); margin-top: 0.25rem;">-</div>
        </div>
        <div class="card" style="text-align: center;">
          <div style="color: var(--text-muted); font-size: 0.85rem;">⚠️ 逾期未還</div>
          <div id="stat-overdue-loans" style="font-size: 2rem; font-weight: bold; color: var(--danger); margin-top: 0.25rem;">-</div>
        </div>
      </div>

      <div class="grid grid-2">
        <div class="card">
          <h3 style="margin-bottom: 1rem;">🔥 最熱門借閱排行 Top 8</h3>
          <div id="top-books-list"></div>
        </div>
        <div class="card">
          <h3 style="margin-bottom: 1rem;">📊 藏書分類分佈</h3>
          <div id="category-chart"></div>
        </div>
      </div>

      <!-- 最近歸還有逾期之歷史紀錄 -->
      <div class="card" style="margin-top: 1.5rem;">
        <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 1rem; flex-wrap: wrap; gap: 0.5rem;">
          <h3 style="margin: 0; color: #b91c1c; display: flex; align-items: center; gap: 0.5rem;">
            <span>⏱️ 最近歸還有逾期之歷史紀錄</span>
          </h3>
          <span style="font-size: 0.85rem; color: var(--text-muted);">顯示最近 10 筆逾期歸還明細</span>
        </div>
        <div class="table-container">
          <table id="recent-overdue-returns-table">
            <thead>
              <tr style="background: #fef2f2;">
                <th>讀者證號 / 姓名</th>
                <th>借閱書籍條碼 / 書名</th>
                <th>借書日期</th>
                <th>應還日期</th>
                <th>歸還日期</th>
                <th>逾期天數</th>
              </tr>
            </thead>
            <tbody>
              <tr><td colspan="6" style="text-align: center; color: var(--text-muted); padding: 1.5rem;">載入中...</td></tr>
            </tbody>
          </table>
        </div>
      </div>
    </section>

    <!-- 9. 系統設定 -->
    <section id="tab-settings" class="tab-content">
      <div class="card" style="max-width: 700px;">
        <h2 style="margin-bottom: 1rem;">⚙️ 圖書館基本資料與標籤設定</h2>
        <div style="margin-bottom: 0.75rem;">
          <label style="font-weight: 600;">學校 / 機構名稱 (印在借書證與書標籤)</label>
          <input type="text" id="set-lib-name" style="margin-top: 0.25rem;">
        </div>
        <div style="margin-bottom: 0.75rem;">
          <label style="font-weight: 600;">電話</label>
          <input type="text" id="set-lib-phone" style="margin-top: 0.25rem;">
        </div>
        <div style="margin-bottom: 0.75rem;">
          <label style="font-weight: 600;">地址</label>
          <input type="text" id="set-lib-address" style="margin-top: 0.25rem;">
        </div>
        <div style="margin-bottom: 0.75rem;">
          <label style="font-weight: 600;">借書證背面「借閱須知規則」</label>
          <textarea id="set-lib-rules" rows="4" style="margin-top: 0.25rem;"></textarea>
        </div>
        <div style="display: flex; gap: 1rem; margin-bottom: 1rem;">
          <div style="flex: 1;">
            <label style="font-weight: 600;">書籍條碼貼紙寬度 (mm)</label>
            <input type="number" id="set-barcode-w" value="48" style="margin-top: 0.25rem;">
          </div>
          <div style="flex: 1;">
            <label style="font-weight: 600;">貼紙高度 (mm)</label>
            <input type="number" id="set-barcode-h" value="40" style="margin-top: 0.25rem;">
          </div>
        </div>
        <div style="margin-bottom: 1.5rem;">
          <label><input type="checkbox" id="set-show-spine" checked> 列印書籍標籤時，一併列印書脊分類背標</label>
        </div>
        <button class="btn btn-primary" onclick="saveSettings()">儲存系統設定</button>

        <hr style="margin: 2rem 0; border: none; border-top: 1px solid var(--border);">

        <h3 style="color: var(--danger); margin-bottom: 0.5rem;">🔄 從舊系統重新移轉資料</h3>
        <p style="color: var(--text-muted); font-size: 0.85rem; margin-bottom: 1rem;">
          讀取本機 data/ 目錄下的 DBF01, Dbf02, Dbf03, Liborrow 等檔案並覆蓋匯入。
        </p>
        <button class="btn btn-outline" style="color: var(--danger); border-color: var(--danger);" onclick="triggerLegacyImport()">重新讀取 data/ 舊資料庫</button>
      </div>
    </section>
  </main>

  <!-- 系統全站頁尾資訊 -->
  <footer class="no-print" style="background: #1e293b; color: #94a3b8; padding: 1.25rem 1.5rem; margin-top: auto; border-top: 1px solid #334155; font-size: 0.85rem;">
    <div style="max-width: 1400px; margin: 0 auto; display: flex; justify-content: space-between; align-items: center; flex-wrap: wrap; gap: 0.75rem;">
      <div style="display: flex; align-items: center; gap: 0.6rem; flex-wrap: wrap;">
        <span style="font-weight: 600; color: #f8fafc;">📚 現代化圖書管理借還系統</span>
        <span style="color: #475569;">|</span>
        <span>版本編號：<code style="background: #0f172a; padding: 0.15rem 0.4rem; border-radius: 0.25rem; color: #38bdf8;">0.3 版</code></span>
        <span style="color: #475569;">|</span>
        <span>編譯日期：<span style="color: #cbd5e1;">2026-11-14</span></span>
      </div>
      <div>
        <span>製作者：<a href="mailto:prolin99@gmail.com" style="color: #60a5fa; text-decoration: none; font-weight: 500;">prolin99@gmail.com</a> 使用 AI 撰寫 Rust 單機版圖書管理系統</span>
      </div>
    </div>
  </footer>

  <!-- 書籍 Modal -->
  <div id="book-modal" class="modal-overlay">
    <div class="modal">
      <div class="modal-header">
        <h3 id="book-modal-title">新增書籍</h3>
        <button class="btn btn-outline btn-sm" onclick="closeModal('book-modal')">✕</button>
      </div>
      <div style="display: flex; gap: 0.5rem; margin-bottom: 0.5rem;">
        <input type="text" id="bm-isbn" placeholder="輸入 ISBN 條碼..." onkeydown="if(event.key==='Enter') lookupIsbnInModal()">
        <button class="btn btn-outline" style="white-space: nowrap;" onclick="lookupIsbnInModal()">🌐 聯網抓取書目</button>
      </div>
      <div id="bm-isbn-source-bar" style="display: none; align-items: center; justify-content: space-between; background: #f0fdf4; border: 1px solid #bbf7d0; padding: 0.4rem 0.6rem; border-radius: 0.375rem; margin-bottom: 0.75rem; font-size: 0.82rem;">
        <div>
          <span style="color: #166534; font-weight: 600;">資料來源：</span>
          <span id="bm-source-name" class="badge badge-info" style="font-size: 0.8rem;">-</span>
        </div>
        <button type="button" id="btn-translate-to-zh" class="btn btn-outline btn-sm" style="display: none; background: white; border-color: #86efac; color: #15803d;" onclick="translateBookModalFields()">
          🔤 一鍵轉成繁體中文
        </button>
      </div>
      <div style="margin-bottom: 0.75rem;">
        <label style="font-size: 0.85rem; font-weight: 600;">書籍條碼 / 登錄號 *</label>
        <input type="text" id="bm-id" required>
      </div>
      <div style="margin-bottom: 0.75rem;">
        <label style="font-size: 0.85rem; font-weight: 600;">書名 *</label>
        <input type="text" id="bm-title" required>
      </div>
      <div style="display: flex; gap: 0.5rem; margin-bottom: 0.75rem;">
        <div style="flex: 1;">
          <label style="font-size: 0.85rem; font-weight: 600;">作者</label>
          <input type="text" id="bm-author">
        </div>
        <div style="flex: 1;">
          <label style="font-size: 0.85rem; font-weight: 600;">出版社</label>
          <input type="text" id="bm-publisher">
        </div>
      </div>
      <div style="display: flex; gap: 0.5rem; margin-bottom: 0.75rem;">
        <div style="flex: 1;">
          <label style="font-size: 0.85rem; font-weight: 600;">分類號</label>
          <input type="text" id="bm-category">
        </div>
        <div style="flex: 1;">
          <label style="font-size: 0.85rem; font-weight: 600;">存放位置/櫃號</label>
          <input type="text" id="bm-location">
        </div>
      </div>
      <div style="margin-bottom: 0.75rem;">
        <label style="font-size: 0.85rem; font-weight: 600;">書籍簡介 (Description)</label>
        <textarea id="bm-description" rows="3"></textarea>
      </div>
      <input type="hidden" id="bm-cover-url">
      <input type="hidden" id="bm-created-at">
      <div style="display: flex; justify-content: flex-end; gap: 0.5rem; margin-top: 1rem;">
        <button class="btn btn-outline" onclick="closeModal('book-modal')">取消</button>
        <button class="btn btn-primary" onclick="saveBookFromModal()">儲存書籍</button>
      </div>
    </div>
  </div>

  <!-- 讀者 Modal -->
  <div id="reader-modal" class="modal-overlay">
    <div class="modal">
      <div class="modal-header">
        <h3 id="reader-modal-title">新增讀者</h3>
        <button class="btn btn-outline btn-sm" onclick="closeModal('reader-modal')">✕</button>
      </div>
      <div style="margin-bottom: 0.75rem;">
        <label style="font-size: 0.85rem; font-weight: 600;">讀者證號 / 學號 *</label>
        <input type="text" id="rm-id" required>
      </div>
      <div style="margin-bottom: 0.75rem;">
        <label style="font-size: 0.85rem; font-weight: 600;">姓名 *</label>
        <input type="text" id="rm-name" required>
      </div>
      <div style="margin-bottom: 0.75rem;">
        <label style="font-size: 0.85rem; font-weight: 600;">身分別 *</label>
        <select id="rm-role-id"></select>
      </div>
      <div style="display: flex; gap: 0.5rem; margin-bottom: 0.75rem;">
        <div style="flex: 1;">
          <label style="font-size: 0.85rem; font-weight: 600;">班級 / 部門</label>
          <input type="text" id="rm-class">
        </div>
        <div style="flex: 1;">
          <label style="font-size: 0.85rem; font-weight: 600;">年級</label>
          <input type="text" id="rm-grade">
        </div>
      </div>
      <div style="display: flex; gap: 0.5rem; margin-bottom: 0.75rem;">
        <div style="flex: 1;">
          <label style="font-size: 0.85rem; font-weight: 600;">電話</label>
          <input type="text" id="rm-phone">
        </div>
        <div style="flex: 1;">
          <label style="font-size: 0.85rem; font-weight: 600;">Email</label>
          <input type="email" id="rm-email">
        </div>
      </div>
      <div style="margin-bottom: 0.75rem;">
        <label style="font-size: 0.85rem; font-weight: 600;">狀態</label>
        <select id="rm-status">
          <option value="active">正常 (啟用)</option>
          <option value="disabled">停用</option>
        </select>
      </div>
      <input type="hidden" id="rm-created-at">
      <div style="display: flex; justify-content: flex-end; gap: 0.5rem; margin-top: 1rem;">
        <button class="btn btn-outline" onclick="closeModal('reader-modal')">取消</button>
        <button class="btn btn-primary" onclick="saveReaderFromModal()">儲存讀者</button>
      </div>
    </div>
  </div>

  <!-- 身分別 Modal -->
  <div id="role-modal" class="modal-overlay">
    <div class="modal">
      <div class="modal-header">
        <h3 id="role-modal-title">身分別借閱規則設定</h3>
        <button class="btn btn-outline btn-sm" onclick="closeModal('role-modal')">✕</button>
      </div>
      <div style="margin-bottom: 0.75rem;">
        <label style="font-size: 0.85rem; font-weight: 600;">身分代號 (如: 01, 02) *</label>
        <input type="text" id="role-m-id" required>
      </div>
      <div style="margin-bottom: 0.75rem;">
        <label style="font-size: 0.85rem; font-weight: 600;">身分名稱 (如: 服務對象, 基金會人員, 教師) *</label>
        <input type="text" id="role-m-name" required>
      </div>
      <div style="display: flex; gap: 0.5rem; margin-bottom: 0.75rem;">
        <div style="flex: 1;">
          <label style="font-size: 0.85rem; font-weight: 600;">可借書籍本數上限 (本) *</label>
          <input type="number" id="role-m-max" value="5" min="1" required>
        </div>
        <div style="flex: 1;">
          <label style="font-size: 0.85rem; font-weight: 600;">每次可借天數 (天) *</label>
          <input type="number" id="role-m-days" value="14" min="1" required>
        </div>
      </div>
      <div style="margin-bottom: 0.75rem;">
        <label style="font-size: 0.85rem; font-weight: 600;">續借天數 (天)</label>
        <input type="number" id="role-m-renew" value="7" min="0">
      </div>
      <div style="margin-bottom: 0.75rem;">
        <label style="font-size: 0.85rem; font-weight: 600;">備註說明</label>
        <input type="text" id="role-m-note">
      </div>
      <div style="display: flex; justify-content: flex-end; gap: 0.5rem; margin-top: 1rem;">
        <button class="btn btn-outline" onclick="closeModal('role-modal')">取消</button>
        <button class="btn btn-primary" onclick="saveRoleFromModal()">儲存身分別</button>
      </div>
    </div>
  </div>

  <script>
    // 全域快取
    let rolesCache = [];
    let currentReader = null;
    let enrichInterval = null;

    // 分頁切換
    function switchTab(tabId) {
      document.querySelectorAll('.tab-content').forEach(el => el.classList.remove('active'));
      document.querySelectorAll('nav button').forEach(el => el.classList.remove('active'));
      const target = document.getElementById('tab-' + tabId);
      if (target) target.classList.add('active');
      event.target.classList.add('active');

      if (tabId === 'active-loans') loadActiveLoans();
      if (tabId === 'books') loadBooks();
      if (tabId === 'readers') loadReaders();
      if (tabId === 'roles') loadRoles();
      if (tabId === 'stats') loadStats();
      if (tabId === 'settings') loadSettings();
    }

    // Modal 輔助
    function openModal(id) { document.getElementById(id).classList.add('active'); }
    function closeModal(id) { document.getElementById(id).classList.remove('active'); }

    // === 動態 SVG 預設封面產生器 ===
    const MORANDI_COLORS = ['#3b82f6', '#10b981', '#f59e0b', '#8b5cf6', '#ec4899', '#06b6d4', '#64748b'];
    function renderBookCover(book) {
      if (book.cover_url && book.cover_url.trim() !== '') {
        return `<img src="${book.cover_url}" class="book-cover" onerror="this.outerHTML=getFallbackSvg('${escapeHtml(book.title)}', '${book.id}')">`;
      }
      return getFallbackSvg(book.title, book.id);
    }

    function getFallbackSvg(title, id) {
      let hash = 0;
      for (let i = 0; i < id.length; i++) hash = id.charCodeAt(i) + ((hash << 5) - hash);
      const color = MORANDI_COLORS[Math.abs(hash) % MORANDI_COLORS.length];
      const shortTitle = (title || '圖書').substring(0, 8);
      return `<div class="cover-fallback" style="background-color: ${color}"><span>📖</span><span>${escapeHtml(shortTitle)}</span></div>`;
    }

    function escapeHtml(text) {
      return (text || '').replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');
    }

    // === 1. 借還書作業 ===
    function clearReaderForBorrow() {
      currentReader = null;
      document.getElementById('borrow-reader-info').style.display = 'none';
      const alertBox = document.getElementById('borrow-reader-overdue-alert');
      if (alertBox) alertBox.style.display = 'none';
      const loansPanel = document.getElementById('borrow-reader-loans-panel');
      if (loansPanel) loansPanel.style.display = 'none';
      const ltbody = document.querySelector('#borrow-reader-loans-table tbody');
      if (ltbody) ltbody.innerHTML = '';
      document.getElementById('bri-name').textContent = '-';
      document.getElementById('bri-role').textContent = '-';
      document.getElementById('bri-active').textContent = '0';
      document.getElementById('bri-max').textContent = '0';
      document.getElementById('bri-days').textContent = '0';
      
      const bookInput = document.getElementById('borrow-book-id');
      bookInput.value = '';
      bookInput.disabled = true;
      bookInput.style.background = '#f1f5f9';
      bookInput.style.color = 'inherit';
      bookInput.style.cursor = 'not-allowed';
      bookInput.placeholder = '⚠️ 請先刷入有效讀者證號...';

      const btn = document.getElementById('btn-submit-borrow');
      if (btn) btn.disabled = true;
    }

    function handleReaderIdInput(val) {
      const rid = (val || '').trim();
      if (!rid) {
        clearReaderForBorrow();
        const err = document.getElementById('borrow-reader-error');
        if (err) err.style.display = 'none';
      }
    }

    async function handleReaderIdChange(val) {
      const rid = (val || '').trim();
      const err = document.getElementById('borrow-reader-error');
      if (!rid) {
        clearReaderForBorrow();
        if (err) err.style.display = 'none';
        return;
      }

      try {
        const res = await fetch(`/api/readers/${encodeURIComponent(rid)}`);
        const data = await res.json();
        if (data.success && data.data) {
          currentReader = data.data;
          document.getElementById('bri-name').textContent = currentReader.name;
          document.getElementById('bri-role').textContent = currentReader.role_name || '一般讀者';
          document.getElementById('bri-max').textContent = currentReader.max_books || 5;
          document.getElementById('bri-days').textContent = currentReader.borrow_days || 14;
          document.getElementById('borrow-reader-info').style.display = 'block';
          if (err) err.style.display = 'none';

          // 取得該讀者目前已借圖書清單
          let activeLoans = [];
          try {
            const lRes = await fetch(`/api/circulation/active?reader_id=${encodeURIComponent(rid)}`);
            const lData = await lRes.json();
            if (lData.success && lData.data) {
              activeLoans = lData.data;
            }
          } catch (le) {
            console.error('取得在借清單失敗', le);
          }

          document.getElementById('bri-active').textContent = activeLoans.length;
          document.getElementById('bri-loans-count-badge').textContent = `${activeLoans.length} 本`;

          // 渲染在借圖書清單
          const ltbody = document.querySelector('#borrow-reader-loans-table tbody');
          ltbody.innerHTML = '';
          const loansPanel = document.getElementById('borrow-reader-loans-panel');
          loansPanel.style.display = 'block';

          if (activeLoans.length === 0) {
            ltbody.innerHTML = '<tr><td colspan="5" style="text-align: center; color: var(--text-muted); padding: 0.75rem;">目前無在借圖書</td></tr>';
          } else {
            activeLoans.forEach(loan => {
              const isOverdue = loan.is_overdue;
              const statusBadge = isOverdue 
                ? `<span class="badge badge-danger" style="font-size: 0.75rem; font-weight: bold;">⚠️ 逾期 ${Math.abs(loan.days_remaining)} 天</span>`
                : `<span class="badge badge-success" style="font-size: 0.75rem;">剩餘 ${loan.days_remaining} 天</span>`;

              const tr = document.createElement('tr');
              tr.style.borderBottom = '1px solid var(--border)';
              tr.innerHTML = `
                <td style="padding: 0.35rem 0.6rem;"><code>${loan.book_id}</code></td>
                <td style="padding: 0.35rem 0.6rem; max-width: 160px; white-space: nowrap; overflow: hidden; text-overflow: ellipsis;" title="${escapeHtml(loan.book_title)}">
                  <strong>${escapeHtml(loan.book_title)}</strong>
                </td>
                <td style="padding: 0.35rem 0.6rem; ${isOverdue ? 'color: var(--danger); font-weight: bold;' : ''}">${loan.due_date}</td>
                <td style="padding: 0.35rem 0.6rem;">${statusBadge}</td>
                <td style="padding: 0.35rem 0.6rem; text-align: center; white-space: nowrap;">
                  <button class="btn btn-outline btn-sm" style="padding: 0.15rem 0.4rem; font-size: 0.75rem;" onclick="quickRenewInBorrow(${loan.id})">續借</button>
                  <button class="btn btn-success btn-sm" style="padding: 0.15rem 0.4rem; font-size: 0.75rem;" onclick="quickReturnInBorrow(${loan.id}, '${escapeHtml(loan.book_title)}')">歸還</button>
                </td>
              `;
              ltbody.appendChild(tr);
            });
          }

          // 逾期檢查
          const overdueLoans = activeLoans.filter(l => l.is_overdue);
          const overdueAlert = document.getElementById('borrow-reader-overdue-alert');
          const bookInput = document.getElementById('borrow-book-id');
          const btn = document.getElementById('btn-submit-borrow');

          if (overdueLoans.length > 0) {
            // 有逾期！顯示警告，禁止借新書
            document.getElementById('bri-overdue-msg').textContent = `該讀者尚有 ${overdueLoans.length} 本圖書逾期未還！依規定暫停借閱權限，在還清逾期書籍前無法借新書。`;
            overdueAlert.style.display = 'block';

            bookInput.value = '';
            bookInput.disabled = true;
            bookInput.style.background = '#fee2e2';
            bookInput.style.color = '#991b1b';
            bookInput.style.cursor = 'not-allowed';
            bookInput.placeholder = '🚫 尚有逾期圖書未還，禁止借新書！';

            if (btn) btn.disabled = true;
          } else {
            // 無逾期
            overdueAlert.style.display = 'none';

            // 檢查是否達到借閱上限
            const maxBooks = currentReader.max_books || 5;
            if (activeLoans.length >= maxBooks) {
              bookInput.value = '';
              bookInput.disabled = true;
              bookInput.style.background = '#fef3c7';
              bookInput.style.color = '#92400e';
              bookInput.style.cursor = 'not-allowed';
              bookInput.placeholder = `⚠️ 已達借閱上限 (${maxBooks} 本)，無法再借新書！`;
              if (btn) btn.disabled = true;
            } else {
              // 正常可借
              bookInput.disabled = false;
              bookInput.style.background = '#ffffff';
              bookInput.style.color = 'inherit';
              bookInput.style.cursor = 'text';
              bookInput.placeholder = '請刷入書籍條碼...';
              bookInput.focus();
              if (btn) btn.disabled = false;
            }
          }
        } else {
          clearReaderForBorrow();
          if (err) {
            err.textContent = `❌ 查無讀者證號 [${rid}]，請確認！`;
            err.style.display = 'block';
          }
          alert(`查無此讀者證號 [${rid}]！讀者資料已清除，請確認後重新輸入。`);
          const rInput = document.getElementById('borrow-reader-id');
          rInput.focus();
          rInput.select();
        }
      } catch (e) {
        clearReaderForBorrow();
        alert('查詢讀者失敗: ' + e);
      }
    }

    async function loadReaderForBorrow() {
      const rid = document.getElementById('borrow-reader-id').value.trim();
      await handleReaderIdChange(rid);
    }

    async function quickReturnInBorrow(loanId, title) {
      if (!confirm(`確定要為書籍《${title}》辦理歸還入庫手續嗎？`)) return;
      const res = await fetch('/api/circulation/return', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ loan_id: loanId })
      });
      const data = await res.json();
      if (data.success) {
        const item = data.data;
        const overdueMsg = item.is_overdue ? ` (⚠️ 逾期 ${item.overdue_days || 0} 天)` : ' (正常歸還)';
        alert(`書籍《${item.book_title}》歸還手續完成！${overdueMsg}`);
        loadReaderForBorrow();
      } else {
        alert(data.message);
      }
    }

    async function quickRenewInBorrow(loanId) {
      const res = await fetch('/api/circulation/renew', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ loan_id: loanId })
      });
      const data = await res.json();
      alert(data.message);
      if (data.success) {
        loadReaderForBorrow();
      }
    }

    async function executeBorrow() {
      const rid = document.getElementById('borrow-reader-id').value.trim();
      const bid = document.getElementById('borrow-book-id').value.trim();
      if (!rid || !bid) {
        alert('請先輸入或刷入讀者證號與書籍條碼！');
        return;
      }
      const res = await fetch('/api/circulation/borrow', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ reader_id: rid, book_id: bid })
      });
      const data = await res.json();
      if (data.success) {
        alert(`借閱成功！書籍《${data.data.book_title}》應還日期為：${data.data.due_date}`);
        document.getElementById('borrow-book-id').value = '';
        await loadReaderForBorrow();
      } else {
        alert(data.message);
      }
    }

    async function executeReturn() {
      const bid = document.getElementById('return-book-id').value.trim();
      if (!bid) return;
      const res = await fetch('/api/circulation/return', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ book_id: bid })
      });
      const data = await res.json();
      const rbox = document.getElementById('return-result-box');
      rbox.style.display = 'block';
      if (data.success) {
        const isOverdue = data.data.is_overdue;
        const overdueDays = data.data.overdue_days || 0;
        rbox.style.background = isOverdue ? '#fee2e2' : '#dcfce7';
        rbox.style.color = isOverdue ? '#991b1b' : '#166534';
        const overdueText = isOverdue 
          ? `<span style="color: var(--danger); font-weight: bold;">⚠️ 逾期歸還 (逾期 ${overdueDays} 天)</span>`
          : `<span style="color: var(--success); font-weight: bold;">正常如期歸還</span>`;
        rbox.innerHTML = `<strong>歸還成功！</strong> 書籍《${escapeHtml(data.data.book_title)}》歸還手續完成。<br>借閱人：${escapeHtml(data.data.reader_name)} (證號: ${data.data.reader_id}) | 應還日期：${data.data.due_date} | 狀態：${overdueText}`;
        document.getElementById('return-book-id').value = '';
        document.getElementById('return-book-id').focus();
        if (currentReader) {
          loadReaderForBorrow();
        }
      } else {
        rbox.style.background = '#fee2e2';
        rbox.style.color = '#991b1b';
        rbox.innerHTML = `<strong>歸還失敗：</strong> ${data.message}`;
      }
    }

    // === 2. 在借與催還 (含多筆分頁與狀態篩選功能) ===
    let activeLoansAllList = [];
    let activeLoansFiltered = [];
    let activeLoansCurrentPage = 1;
    let activeLoansPageSize = 20;

    async function loadActiveLoans() {
      const res = await fetch('/api/circulation/active');
      const data = await res.json();
      if (data.success && data.data) {
        activeLoansAllList = data.data;
      } else {
        activeLoansAllList = [];
      }
      filterActiveLoans();
    }

    function filterActiveLoans() {
      const q = (document.getElementById('active-loans-search')?.value || '').trim().toLowerCase();
      const filter = document.getElementById('active-loans-status-filter')?.value || '';

      activeLoansFiltered = activeLoansAllList.filter(item => {
        if (filter === 'overdue' && !item.is_overdue) return false;
        if (filter === 'normal' && item.is_overdue) return false;
        if (q) {
          const match = (item.book_title || '').toLowerCase().includes(q)
            || (item.book_id || '').toLowerCase().includes(q)
            || (item.reader_name || '').toLowerCase().includes(q)
            || (item.reader_id || '').toLowerCase().includes(q);
          if (!match) return false;
        }
        return true;
      });

      activeLoansCurrentPage = 1;
      renderActiveLoansTable();
    }

    function changeActiveLoansPageSize(val) {
      activeLoansPageSize = parseInt(val) || 0;
      activeLoansCurrentPage = 1;
      renderActiveLoansTable();
    }

    function gotoActiveLoansPage(page) {
      const totalPages = activeLoansPageSize === 0 ? 1 : Math.max(1, Math.ceil(activeLoansFiltered.length / activeLoansPageSize));
      activeLoansCurrentPage = Math.max(1, Math.min(page, totalPages));
      renderActiveLoansTable();
    }

    function prevActiveLoansPage() {
      if (activeLoansCurrentPage > 1) {
        activeLoansCurrentPage--;
        renderActiveLoansTable();
      }
    }

    function nextActiveLoansPage() {
      const totalPages = activeLoansPageSize === 0 ? 1 : Math.max(1, Math.ceil(activeLoansFiltered.length / activeLoansPageSize));
      if (activeLoansCurrentPage < totalPages) {
        activeLoansCurrentPage++;
        renderActiveLoansTable();
      }
    }

    function gotoActiveLoansLastPage() {
      const totalPages = activeLoansPageSize === 0 ? 1 : Math.max(1, Math.ceil(activeLoansFiltered.length / activeLoansPageSize));
      activeLoansCurrentPage = totalPages;
      renderActiveLoansTable();
    }

    function renderActiveLoansTable() {
      const total = activeLoansFiltered.length;
      const overdueCount = activeLoansFiltered.filter(i => i.is_overdue).length;
      const totalPages = activeLoansPageSize === 0 ? 1 : Math.max(1, Math.ceil(total / activeLoansPageSize));
      if (activeLoansCurrentPage > totalPages) activeLoansCurrentPage = totalPages;

      let displayList = [];
      let startIdx = 0, endIdx = 0;
      if (activeLoansPageSize === 0) {
        displayList = activeLoansFiltered;
        startIdx = total > 0 ? 1 : 0;
        endIdx = total;
      } else {
        const start = (activeLoansCurrentPage - 1) * activeLoansPageSize;
        const end = Math.min(start + activeLoansPageSize, total);
        displayList = activeLoansFiltered.slice(start, end);
        startIdx = total > 0 ? start + 1 : 0;
        endIdx = end;
      }

      const summaryText = `共 ${total} 筆在借記錄 (逾期催還 ${overdueCount} 筆)，顯示第 ${startIdx} - ${endIdx} 筆`;
      const pageText = `第 ${activeLoansCurrentPage} / ${totalPages} 頁`;

      const sTop = document.getElementById('active-loans-page-summary');
      const sBtm = document.getElementById('active-loans-page-summary-bottom');
      if (sTop) sTop.textContent = summaryText;
      if (sBtm) sBtm.textContent = summaryText;

      const iTop = document.getElementById('active-loans-page-indicator');
      const iBtm = document.getElementById('active-loans-page-indicator-bottom');
      if (iTop) iTop.textContent = pageText;
      if (iBtm) iBtm.textContent = pageText;

      const bFirst = document.getElementById('btn-loans-first');
      const bPrev = document.getElementById('btn-loans-prev');
      const bNext = document.getElementById('btn-loans-next');
      const bLast = document.getElementById('btn-loans-last');
      if (bFirst) bFirst.disabled = (activeLoansCurrentPage <= 1);
      if (bPrev) bPrev.disabled = (activeLoansCurrentPage <= 1);
      if (bNext) bNext.disabled = (activeLoansCurrentPage >= totalPages);
      if (bLast) bLast.disabled = (activeLoansCurrentPage >= totalPages);

      const tbody = document.querySelector('#active-loans-table tbody');
      tbody.innerHTML = '';
      if (displayList.length === 0) {
        tbody.innerHTML = '<tr><td colspan="9" style="text-align: center; color: var(--text-muted); padding: 2.5rem;">無符合條件之借閱記錄</td></tr>';
        return;
      }

      displayList.forEach(item => {
        const isOverdue = item.is_overdue;
        const statusBadge = isOverdue 
          ? `<span class="badge badge-danger">逾期 ${Math.abs(item.days_remaining)} 天</span>`
          : `<span class="badge badge-success">剩餘 ${item.days_remaining} 天</span>`;

        const tr = document.createElement('tr');
        tr.innerHTML = `
          <td><code>${item.book_id}</code></td>
          <td><strong>${escapeHtml(item.book_title)}</strong></td>
          <td>${item.reader_id}</td>
          <td>${escapeHtml(item.reader_name)}</td>
          <td>${escapeHtml(item.role_name || '-')}</td>
          <td>${item.borrow_date}</td>
          <td style="${isOverdue ? 'color: var(--danger); font-weight: bold;' : ''}">${item.due_date}</td>
          <td>${statusBadge}</td>
          <td>
            <button class="btn btn-outline btn-sm" onclick="renewLoan(${item.id})">續借</button>
            <button class="btn btn-success btn-sm" onclick="returnLoanById(${item.id})">歸還</button>
          </td>
        `;
        tbody.appendChild(tr);
      });
    }

    async function renewLoan(loanId) {
      const res = await fetch('/api/circulation/renew', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ loan_id: loanId })
      });
      const data = await res.json();
      alert(data.message);
      if (data.success) loadActiveLoans();
    }

    async function returnLoanById(loanId) {
      const res = await fetch('/api/circulation/return', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ loan_id: loanId })
      });
      const data = await res.json();
      if (data.success) {
        const item = data.data;
        const overdueMsg = item.is_overdue ? ` (⚠️ 逾期 ${item.overdue_days || 0} 天)` : ' (正常歸還)';
        alert(`書籍《${item.book_title}》歸還手續完成！${overdueMsg}`);
        loadActiveLoans();
      } else {
        alert(data.message);
      }
    }

    // === 3. 書籍管理 (含多筆分頁功能) ===
    let booksAllList = [];
    let booksCurrentPage = 1;
    let booksPageSize = 20;

    async function loadBooks() {
      const q = document.getElementById('books-search').value.trim();
      const st = document.getElementById('books-status-filter').value;
      const res = await fetch(`/api/books?q=${encodeURIComponent(q)}&status=${encodeURIComponent(st)}`);
      const data = await res.json();
      if (data.success && data.data) {
        booksAllList = data.data;
        booksCurrentPage = 1; // 重新搜尋時回到第 1 頁
        renderBooksTable();
      } else {
        booksAllList = [];
        renderBooksTable();
      }
    }

    function changeBooksPageSize(val) {
      booksPageSize = parseInt(val) || 0;
      booksCurrentPage = 1;
      renderBooksTable();
    }

    function gotoBooksPage(page) {
      const totalPages = booksPageSize === 0 ? 1 : Math.max(1, Math.ceil(booksAllList.length / booksPageSize));
      booksCurrentPage = Math.max(1, Math.min(page, totalPages));
      renderBooksTable();
    }

    function prevBooksPage() {
      if (booksCurrentPage > 1) {
        booksCurrentPage--;
        renderBooksTable();
      }
    }

    function nextBooksPage() {
      const totalPages = booksPageSize === 0 ? 1 : Math.max(1, Math.ceil(booksAllList.length / booksPageSize));
      if (booksCurrentPage < totalPages) {
        booksCurrentPage++;
        renderBooksTable();
      }
    }

    function gotoBooksLastPage() {
      const totalPages = booksPageSize === 0 ? 1 : Math.max(1, Math.ceil(booksAllList.length / booksPageSize));
      booksCurrentPage = totalPages;
      renderBooksTable();
    }

    function renderBooksTable() {
      const total = booksAllList.length;
      const totalPages = booksPageSize === 0 ? 1 : Math.max(1, Math.ceil(total / booksPageSize));
      if (booksCurrentPage > totalPages) booksCurrentPage = totalPages;

      let displayList = [];
      let startIdx = 0;
      let endIdx = 0;

      if (booksPageSize === 0) {
        displayList = booksAllList;
        startIdx = total > 0 ? 1 : 0;
        endIdx = total;
      } else {
        const start = (booksCurrentPage - 1) * booksPageSize;
        const end = Math.min(start + booksPageSize, total);
        displayList = booksAllList.slice(start, end);
        startIdx = total > 0 ? start + 1 : 0;
        endIdx = end;
      }

      // 更新分頁資訊標籤 (頂部與底部)
      const summaryText = `共 ${total} 本書籍，目前顯示第 ${startIdx} - ${endIdx} 筆`;
      const pageIndicatorText = `第 ${booksCurrentPage} / ${totalPages} 頁`;

      const sumTop = document.getElementById('books-page-summary');
      if (sumTop) sumTop.textContent = summaryText;
      const sumBtm = document.getElementById('books-page-summary-bottom');
      if (sumBtm) sumBtm.textContent = summaryText;

      const indTop = document.getElementById('books-page-indicator');
      if (indTop) indTop.textContent = pageIndicatorText;
      const indBtm = document.getElementById('books-page-indicator-bottom');
      if (indBtm) indBtm.textContent = pageIndicatorText;

      // 按鈕啟用/停用狀態
      const btnFirst = document.getElementById('btn-books-first');
      const btnPrev = document.getElementById('btn-books-prev');
      const btnNext = document.getElementById('btn-books-next');
      const btnLast = document.getElementById('btn-books-last');

      if (btnFirst) btnFirst.disabled = (booksCurrentPage <= 1);
      if (btnPrev) btnPrev.disabled = (booksCurrentPage <= 1);
      if (btnNext) btnNext.disabled = (booksCurrentPage >= totalPages);
      if (btnLast) btnLast.disabled = (booksCurrentPage >= totalPages);

      const tbody = document.querySelector('#books-table tbody');
      tbody.innerHTML = '';

      if (displayList.length === 0) {
        tbody.innerHTML = '<tr><td colspan="11" style="text-align: center; color: var(--text-muted); padding: 2.5rem;">無符合搜尋條件之書籍資料</td></tr>';
        return;
      }

      displayList.forEach(book => {
        const statusBadge = book.status === 'available'
          ? '<span class="badge badge-success">在館</span>'
          : `<span class="badge badge-danger">借出 (${escapeHtml(book.current_borrower?.reader_name || '')})</span>`;

        const tr = document.createElement('tr');
        tr.innerHTML = `
          <td>${renderBookCover(book)}</td>
          <td><code>${book.id}</code></td>
          <td><strong>${escapeHtml(book.title)}</strong></td>
          <td>${escapeHtml(book.author)}</td>
          <td>${escapeHtml(book.publisher)}</td>
          <td>${book.isbn || '-'}</td>
          <td>${book.category || '-'}</td>
          <td>${book.location || '-'}</td>
          <td>${statusBadge}</td>
          <td>${book.borrow_count}</td>
          <td>
            <button class="btn btn-outline btn-sm" onclick='editBook(${JSON.stringify(book)})'>編輯</button>
            <button class="btn btn-outline btn-sm" style="color: var(--danger);" onclick="deleteBook('${book.id}')">刪除</button>
          </td>
        `;
        tbody.appendChild(tr);
      });
    }

    async function openBookModal(book = null) {
      document.getElementById('book-modal-title').textContent = book ? '編輯書籍' : '新增書籍';
      document.getElementById('bm-id').readOnly = !!book;
      document.getElementById('bm-title').value = book ? book.title : '';
      document.getElementById('bm-author').value = book ? book.author : '';
      document.getElementById('bm-publisher').value = book ? book.publisher : '';
      document.getElementById('bm-category').value = book ? book.category : '';
      document.getElementById('bm-location').value = book ? book.location : '';
      document.getElementById('bm-isbn').value = book ? book.isbn : '';
      document.getElementById('bm-description').value = book ? book.description : '';
      document.getElementById('bm-cover-url').value = book ? book.cover_url : '';
      document.getElementById('bm-created-at').value = book ? book.created_at : '';

      // 重設來源標籤與翻譯按鈕
      const srcBar = document.getElementById('bm-isbn-source-bar');
      if (srcBar) srcBar.style.display = 'none';
      const transBtn = document.getElementById('btn-translate-to-zh');
      if (transBtn) transBtn.style.display = 'none';

      if (book) {
        document.getElementById('bm-id').value = book.id;
      } else {
        document.getElementById('bm-id').value = '讀取預設條碼中...';
        try {
          const res = await fetch('/api/books/next-id');
          const data = await res.json();
          if (data.success && data.data) {
            document.getElementById('bm-id').value = data.data;
          } else {
            document.getElementById('bm-id').value = '';
          }
        } catch (e) {
          document.getElementById('bm-id').value = '';
        }
      }

      openModal('book-modal');
    }

    function editBook(book) { openBookModal(book); }

    async function lookupIsbnInModal() {
      const isbn = document.getElementById('bm-isbn').value.trim();
      if (!isbn) { alert('請先輸入 ISBN！'); return; }
      
      const srcBar = document.getElementById('bm-isbn-source-bar');
      const srcName = document.getElementById('bm-source-name');
      const transBtn = document.getElementById('btn-translate-to-zh');

      const res = await fetch('/api/books/lookup-isbn', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ isbn })
      });
      const data = await res.json();
      if (data.success && data.data) {
        const item = data.data;
        document.getElementById('bm-title').value = item.title || document.getElementById('bm-title').value;
        document.getElementById('bm-author').value = item.author || document.getElementById('bm-author').value;
        document.getElementById('bm-publisher').value = item.publisher || document.getElementById('bm-publisher').value;
        document.getElementById('bm-description').value = item.description || document.getElementById('bm-description').value;
        if (item.cover_url) document.getElementById('bm-cover-url').value = item.cover_url;

        // 顯示來源徽章與翻譯按鈕
        if (srcBar && srcName) {
          srcName.textContent = item.source || '網路資料庫';
          srcBar.style.display = 'flex';
        }
        if (transBtn) {
          // 如果書名、作者或出版社包含非中文，顯示一鍵更換中文按鈕
          const hasNonChinese = !item.is_chinese 
            || (item.author && !/[\u4e00-\u9fa5]/.test(item.author))
            || (item.publisher && !/[\u4e00-\u9fa5]/.test(item.publisher));

          transBtn.style.display = 'inline-block';
          if (hasNonChinese) {
            transBtn.textContent = '🔤 書名/作者/出版社一鍵轉成中文';
            transBtn.classList.add('btn-primary');
            transBtn.classList.remove('btn-outline');
          } else {
            transBtn.textContent = '🔤 一鍵轉繁體中文';
            transBtn.classList.remove('btn-primary');
            transBtn.classList.add('btn-outline');
          }
        }

        alert(data.message || `已由【${item.source}】成功獲取書目資訊！`);
      } else {
        if (srcBar) srcBar.style.display = 'none';
        alert(data.message);
      }
    }

    async function translateBookModalFields() {
      const title = document.getElementById('bm-title').value.trim();
      const author = document.getElementById('bm-author').value.trim();
      const publisher = document.getElementById('bm-publisher').value.trim();
      const desc = document.getElementById('bm-description').value.trim();
      const btn = document.getElementById('btn-translate-to-zh');
      if (!title && !author && !publisher && !desc) {
        alert('書名、作者、出版社與簡介均為空，無需更換！');
        return;
      }
      
      const origText = btn ? btn.textContent : '';
      if (btn) {
        btn.textContent = '⏳ 正在轉成中文...';
        btn.disabled = true;
      }

      try {
        // 1. 翻譯書名
        if (title && !/[\u4e00-\u9fa5]/.test(title)) {
          const tRes = await fetch('/api/system/translate', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ text: title })
          });
          const tData = await tRes.json();
          if (tData.success && tData.data && tData.data.translated) {
            document.getElementById('bm-title').value = tData.data.translated;
          }
        }

        // 2. 翻譯作者 (例如 "Moriya, Hiroshi" -> "森谷浩史", "Shouwu Yang" -> "楊守愚")
        if (author && !/[\u4e00-\u9fa5]/.test(author)) {
          const aRes = await fetch('/api/system/translate', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ text: author })
          });
          const aData = await aRes.json();
          if (aData.success && aData.data && aData.data.translated) {
            document.getElementById('bm-author').value = aData.data.translated;
          }
        }

        // 3. 翻譯出版社 (例如 "Da bu wen hua" -> "大步文化")
        if (publisher && !/[\u4e00-\u9fa5]/.test(publisher)) {
          const pRes = await fetch('/api/system/translate', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ text: publisher })
          });
          const pData = await pRes.json();
          if (pData.success && pData.data && pData.data.translated) {
            document.getElementById('bm-publisher').value = pData.data.translated;
          }
        }

        // 4. 翻譯簡介
        if (desc && !/[\u4e00-\u9fa5]/.test(desc)) {
          const dRes = await fetch('/api/system/translate', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ text: desc })
          });
          const dData = await dRes.json();
          if (dData.success && dData.data && dData.data.translated) {
            document.getElementById('bm-description').value = dData.data.translated;
          }
        }

        alert('書名、作者、出版社已成功更換為繁體中文！');
      } catch (e) {
        alert('更換中文發生錯誤: ' + e);
      } finally {
        if (btn) {
          btn.textContent = '✅ 已轉為中文';
          btn.disabled = false;
        }
      }
    }

    async function saveBookFromModal() {
      const book = {
        id: document.getElementById('bm-id').value.trim(),
        title: document.getElementById('bm-title').value.trim(),
        author: document.getElementById('bm-author').value.trim(),
        publisher: document.getElementById('bm-publisher').value.trim(),
        pub_year: '',
        isbn: document.getElementById('bm-isbn').value.trim(),
        category: document.getElementById('bm-category').value.trim(),
        location: document.getElementById('bm-location').value.trim(),
        price: 0,
        description: document.getElementById('bm-description').value.trim(),
        cover_url: document.getElementById('bm-cover-url').value,
        status: 'available',
        borrow_count: 0,
        enrichment_status: 'none',
        note: '',
        created_at: document.getElementById('bm-created-at').value,
        current_borrower: null,
      };
      if (!book.id || !book.title) { alert('請填寫書籍條碼與書名！'); return; }
      const res = await fetch('/api/books', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(book)
      });
      const data = await res.json();
      if (data.success) {
        closeModal('book-modal');
        loadBooks();
      } else {
        alert(data.message);
      }
    }

    async function deleteBook(id) {
      if (!confirm(`確定要刪除書籍 [${id}] 嗎？`)) return;
      const res = await fetch(`/api/books/${encodeURIComponent(id)}`, { method: 'DELETE' });
      const data = await res.json();
      alert(data.message);
      if (data.success) loadBooks();
    }

    async function uploadBooksFile(input) {
      if (!input.files || input.files.length === 0) return;
      const formData = new FormData();
      formData.append('file', input.files[0]);
      const res = await fetch('/api/import/books', { method: 'POST', body: formData });
      const data = await res.json();
      alert(data.message);
      input.value = '';
      if (data.success) loadBooks();
    }

    // === 4. 讀者管理 (含多筆分頁功能) ===
    let readersAllList = [];
    let readersCurrentPage = 1;
    let readersPageSize = 20;

    async function loadReaders() {
      const q = document.getElementById('readers-search').value.trim();
      const res = await fetch(`/api/readers?q=${encodeURIComponent(q)}`);
      const data = await res.json();
      if (data.success && data.data) {
        readersAllList = data.data;
        readersCurrentPage = 1;
        renderReadersTable();
      } else {
        readersAllList = [];
        renderReadersTable();
      }
    }

    function changeReadersPageSize(val) {
      readersPageSize = parseInt(val) || 0;
      readersCurrentPage = 1;
      renderReadersTable();
    }

    function gotoReadersPage(page) {
      const totalPages = readersPageSize === 0 ? 1 : Math.max(1, Math.ceil(readersAllList.length / readersPageSize));
      readersCurrentPage = Math.max(1, Math.min(page, totalPages));
      renderReadersTable();
    }

    function prevReadersPage() {
      if (readersCurrentPage > 1) {
        readersCurrentPage--;
        renderReadersTable();
      }
    }

    function nextReadersPage() {
      const totalPages = readersPageSize === 0 ? 1 : Math.max(1, Math.ceil(readersAllList.length / readersPageSize));
      if (readersCurrentPage < totalPages) {
        readersCurrentPage++;
        renderReadersTable();
      }
    }

    function gotoReadersLastPage() {
      const totalPages = readersPageSize === 0 ? 1 : Math.max(1, Math.ceil(readersAllList.length / readersPageSize));
      readersCurrentPage = totalPages;
      renderReadersTable();
    }

    function renderReadersTable() {
      const total = readersAllList.length;
      const totalPages = readersPageSize === 0 ? 1 : Math.max(1, Math.ceil(total / readersPageSize));
      if (readersCurrentPage > totalPages) readersCurrentPage = totalPages;

      let displayList = [];
      let startIdx = 0;
      let endIdx = 0;

      if (readersPageSize === 0) {
        displayList = readersAllList;
        startIdx = total > 0 ? 1 : 0;
        endIdx = total;
      } else {
        const start = (readersCurrentPage - 1) * readersPageSize;
        const end = Math.min(start + readersPageSize, total);
        displayList = readersAllList.slice(start, end);
        startIdx = total > 0 ? start + 1 : 0;
        endIdx = end;
      }

      const summaryText = `共 ${total} 位讀者，目前顯示第 ${startIdx} - ${endIdx} 筆`;
      const pageIndicatorText = `第 ${readersCurrentPage} / ${totalPages} 頁`;

      const sumTop = document.getElementById('readers-page-summary');
      if (sumTop) sumTop.textContent = summaryText;
      const sumBtm = document.getElementById('readers-page-summary-bottom');
      if (sumBtm) sumBtm.textContent = summaryText;

      const indTop = document.getElementById('readers-page-indicator');
      if (indTop) indTop.textContent = pageIndicatorText;
      const indBtm = document.getElementById('readers-page-indicator-bottom');
      if (indBtm) indBtm.textContent = pageIndicatorText;

      const btnFirst = document.getElementById('btn-readers-first');
      const btnPrev = document.getElementById('btn-readers-prev');
      const btnNext = document.getElementById('btn-readers-next');
      const btnLast = document.getElementById('btn-readers-last');

      if (btnFirst) btnFirst.disabled = (readersCurrentPage <= 1);
      if (btnPrev) btnPrev.disabled = (readersCurrentPage <= 1);
      if (btnNext) btnNext.disabled = (readersCurrentPage >= totalPages);
      if (btnLast) btnLast.disabled = (readersCurrentPage >= totalPages);

      const tbody = document.querySelector('#readers-table tbody');
      tbody.innerHTML = '';

      if (displayList.length === 0) {
        tbody.innerHTML = '<tr><td colspan="9" style="text-align: center; color: var(--text-muted); padding: 2.5rem;">無符合搜尋條件之讀者資料</td></tr>';
        return;
      }

      displayList.forEach(r => {
        const statusBadge = r.status === 'active'
          ? '<span class="badge badge-success">正常</span>'
          : '<span class="badge badge-danger">停用</span>';

        const tr = document.createElement('tr');
        tr.innerHTML = `
          <td><code>${r.id}</code></td>
          <td><strong>${escapeHtml(r.name)}</strong></td>
          <td><span class="badge badge-info">${escapeHtml(r.role_name || '-')}</span></td>
          <td>${r.max_books || '-'} 本</td>
          <td>${r.borrow_days || '-'} 天</td>
          <td>${escapeHtml(r.class_name || '-')}</td>
          <td>${r.active_loans_count || 0} 本</td>
          <td>${statusBadge}</td>
          <td>
            <button class="btn btn-outline btn-sm" onclick='editReader(${JSON.stringify(r)})'>編輯</button>
            <button class="btn btn-outline btn-sm" style="color: var(--danger);" onclick="deleteReader('${r.id}')">刪除</button>
          </td>
        `;
        tbody.appendChild(tr);
      });
    }

    async function loadRolesForSelect(selectedId = '') {
      await loadRoles();
      const sel = document.getElementById('rm-role-id');
      sel.innerHTML = '';
      rolesCache.forEach(role => {
        const opt = document.createElement('option');
        opt.value = role.id;
        opt.textContent = `${role.name} (可借${role.max_books}本 / ${role.borrow_days}天)`;
        if (role.id === selectedId) opt.selected = true;
        sel.appendChild(opt);
      });
    }

    async function openReaderModal(reader = null) {
      document.getElementById('reader-modal-title').textContent = reader ? '編輯讀者' : '新增讀者';
      document.getElementById('rm-id').readOnly = !!reader;
      document.getElementById('rm-name').value = reader ? reader.name : '';
      document.getElementById('rm-class').value = reader ? reader.class_name : '';
      document.getElementById('rm-grade').value = reader ? reader.grade : '';
      document.getElementById('rm-phone').value = reader ? reader.phone : '';
      document.getElementById('rm-email').value = reader ? reader.email : '';
      document.getElementById('rm-status').value = reader ? reader.status : 'active';
      document.getElementById('rm-created-at').value = reader ? reader.created_at : '';
      loadRolesForSelect(reader ? reader.role_id : '01');

      if (reader) {
        document.getElementById('rm-id').value = reader.id;
      } else {
        document.getElementById('rm-id').value = '讀取預設證號中...';
        try {
          const res = await fetch('/api/readers/next-id');
          const data = await res.json();
          if (data.success && data.data) {
            document.getElementById('rm-id').value = data.data;
          } else {
            document.getElementById('rm-id').value = '';
          }
        } catch (e) {
          document.getElementById('rm-id').value = '';
        }
      }

      openModal('reader-modal');
    }

    function editReader(reader) { openReaderModal(reader); }

    async function saveReaderFromModal() {
      const reader = {
        id: document.getElementById('rm-id').value.trim(),
        name: document.getElementById('rm-name').value.trim(),
        role_id: document.getElementById('rm-role-id').value,
        role_name: null,
        max_books: null,
        borrow_days: null,
        class_name: document.getElementById('rm-class').value.trim(),
        grade: document.getElementById('rm-grade').value.trim(),
        phone: document.getElementById('rm-phone').value.trim(),
        email: document.getElementById('rm-email').value.trim(),
        status: document.getElementById('rm-status').value,
        note: '',
        created_at: document.getElementById('rm-created-at').value,
        active_loans_count: 0,
      };
      if (!reader.id || !reader.name) { alert('請填寫讀者證號與姓名！'); return; }
      const res = await fetch('/api/readers', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(reader)
      });
      const data = await res.json();
      if (data.success) {
        closeModal('reader-modal');
        loadReaders();
      } else {
        alert(data.message);
      }
    }

    async function deleteReader(id) {
      if (!confirm(`確定要刪除讀者 [${id}] 嗎？`)) return;
      const res = await fetch(`/api/readers/${encodeURIComponent(id)}`, { method: 'DELETE' });
      const data = await res.json();
      alert(data.message);
      if (data.success) loadReaders();
    }

    async function uploadReadersFile(input) {
      if (!input.files || input.files.length === 0) return;
      const formData = new FormData();
      formData.append('file', input.files[0]);
      const res = await fetch('/api/import/readers', { method: 'POST', body: formData });
      const data = await res.json();
      alert(data.message);
      input.value = '';
      if (data.success) loadReaders();
    }

    // === 5. 身分別設定 ===
    async function loadRoles() {
      const res = await fetch('/api/roles');
      const data = await res.json();
      if (data.success && data.data) {
        rolesCache = data.data;
        const tbody = document.querySelector('#roles-table tbody');
        tbody.innerHTML = '';
        rolesCache.forEach(r => {
          const tr = document.createElement('tr');
          tr.innerHTML = `
            <td><code>${r.id}</code></td>
            <td><strong>${escapeHtml(r.name)}</strong></td>
            <td><strong style="color: var(--primary);">${r.max_books}</strong> 本</td>
            <td>${r.borrow_days} 天</td>
            <td>${r.renew_days} 天</td>
            <td>${escapeHtml(r.note || '-')}</td>
            <td>
              <button class="btn btn-outline btn-sm" onclick='editRole(${JSON.stringify(r)})'>設定</button>
              <button class="btn btn-outline btn-sm" style="color: var(--danger);" onclick="deleteRole('${r.id}')">刪除</button>
            </td>
          `;
          tbody.appendChild(tr);
        });
      }
    }

    function openRoleModal(role = null) {
      document.getElementById('role-modal-title').textContent = role ? '修改身分別規則' : '新增身分別';
      document.getElementById('role-m-id').value = role ? role.id : '';
      document.getElementById('role-m-id').readOnly = !!role;
      document.getElementById('role-m-name').value = role ? role.name : '';
      document.getElementById('role-m-max').value = role ? role.max_books : 5;
      document.getElementById('role-m-days').value = role ? role.borrow_days : 14;
      document.getElementById('role-m-renew').value = role ? role.renew_days : 7;
      document.getElementById('role-m-note').value = role ? role.note : '';
      openModal('role-modal');
    }

    function editRole(role) { openRoleModal(role); }

    async function saveRoleFromModal() {
      const role = {
        id: document.getElementById('role-m-id').value.trim(),
        name: document.getElementById('role-m-name').value.trim(),
        max_books: parseInt(document.getElementById('role-m-max').value) || 5,
        borrow_days: parseInt(document.getElementById('role-m-days').value) || 14,
        renew_days: parseInt(document.getElementById('role-m-renew').value) || 7,
        note: document.getElementById('role-m-note').value.trim(),
      };
      if (!role.id || !role.name) { alert('請填寫身分代號與名稱！'); return; }
      const res = await fetch('/api/roles', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(role)
      });
      const data = await res.json();
      if (data.success) {
        closeModal('role-modal');
        loadRoles();
      } else {
        alert(data.message);
      }
    }

    async function deleteRole(id) {
      if (!confirm(`確定要刪除身分別 [${id}] 嗎？`)) return;
      const res = await fetch(`/api/roles/${encodeURIComponent(id)}`, { method: 'DELETE' });
      const data = await res.json();
      alert(data.message);
      if (data.success) loadRoles();
    }

    // === 6. 網路補齊 ===
    async function startEnrichment() {
      const rpm = parseInt(document.getElementById('enrich-rpm').value) || 20;
      const res = await fetch('/api/enrichment/start', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          requests_per_minute: rpm,
          max_requests_per_hour: 500,
          jitter_ms: 1000,
          update_cover: document.getElementById('enrich-cover').checked,
          update_desc: document.getElementById('enrich-desc').checked,
          update_meta: true,
          overwrite_existing: false,
        })
      });
      const data = await res.json();
      alert(data.message);
      if (data.success) pollEnrichment();
    }

    async function stopEnrichment() {
      const res = await fetch('/api/enrichment/stop', { method: 'POST' });
      const data = await res.json();
      alert(data.message);
    }

    function pollEnrichment() {
      if (enrichInterval) clearInterval(enrichInterval);
      enrichInterval = setInterval(async () => {
        const res = await fetch('/api/enrichment/progress');
        const data = await res.json();
        if (data.success && data.data) {
          const p = data.data;
          const pct = p.total > 0 ? Math.round((p.processed / p.total) * 100) : 0;
          document.getElementById('enrich-progress-bar').style.width = pct + '%';
          document.getElementById('enrich-progress-percent').textContent = pct + '%';
          document.getElementById('enrich-progress-count').textContent = `${p.processed} / ${p.total}`;
          document.getElementById('enrich-status-msg').textContent = p.last_message || '執行中...';
          if (!p.is_running && p.total > 0 && p.processed >= p.total) {
            clearInterval(enrichInterval);
          }
        }
      }, 1500);
    }

    // === 7. 批次列印 (借書證 & 書籍標籤) ===
    // 內建純 SVG 條碼產生器 (Code 39)
    const CODE39_MAP = {
      '0': '000110100', '1': '100100001', '2': '001100001', '3': '101100000', '4': '000110001',
      '5': '100110000', '6': '001110000', '7': '000100101', '8': '100100100', '9': '001100100',
      'A': '100001001', 'B': '001001001', 'C': '101001000', 'D': '000011001', 'E': '100011000',
      'F': '001011000', 'G': '000001101', 'H': '100001100', 'I': '001001100', 'J': '000011100',
      'K': '100000011', 'L': '001000011', 'M': '101000010', 'N': '000010011', 'O': '100010010',
      'P': '001010010', 'Q': '000000111', 'R': '100000110', 'S': '001000110', 'T': '000010110',
      'U': '110000001', 'V': '011000001', 'W': '111000000', 'X': '010010001', 'Y': '110010000',
      'Z': '011010000', '-': '010000101', '.': '110000100', ' ': '011000100', '$': '010101000',
      '/': '010100010', '+': '010001010', '%': '000101010', '*': '010010100'
    };

    function generateCode39Svg(text, height = 36) {
      const code = '*' + (text || '').toUpperCase() + '*';
      let x = 0;
      let bars = '';
      for (let i = 0; i < code.length; i++) {
        const char = code[i];
        const pattern = CODE39_MAP[char] || CODE39_MAP[' '];
        for (let j = 0; j < 9; j++) {
          const isBar = (j % 2 === 0);
          const isWide = (pattern[j] === '1');
          const width = isWide ? 3 : 1;
          if (isBar) {
            bars += `<rect x="${x}" y="0" width="${width}" height="${height}" fill="black" />`;
          }
          x += width;
        }
        x += 2; // 間隔
      }
      return `<svg viewBox="0 0 ${x} ${height}" style="height: ${height}px; max-width: 100%; display: block; margin: 0 auto;">${bars}</svg>`;
    }

    // 列印範圍過濾輔助函數
    function filterPrintItems(items, idKey, searchKeys = []) {
      const start = (document.getElementById('print-range-start').value || '').trim();
      const end = (document.getElementById('print-range-end').value || '').trim();
      const kw = (document.getElementById('print-range-filter').value || '').trim().toLowerCase();
      const limit = parseInt(document.getElementById('print-range-limit').value) || 0;

      // 提取字串尾端的數字，若無數字則回傳 NaN
      function extractTrailingNum(str) {
        const m = (str || '').match(/(\d+)$/);
        return m ? parseInt(m[1], 10) : NaN;
      }

      let filtered = items.filter(item => {
        const id = (item[idKey] || '').trim();
        
        // 1. 起訖編號篩選
        if (start || end) {
          const itemNum = extractTrailingNum(id);
          const startNum = start ? extractTrailingNum(start) : NaN;
          const endNum = end ? extractTrailingNum(end) : NaN;

          if (!isNaN(itemNum) && (!isNaN(startNum) || !isNaN(endNum))) {
            if (!isNaN(startNum) && itemNum < startNum) return false;
            if (!isNaN(endNum) && itemNum > endNum) return false;
          } else {
            // 文字字典序比對
            if (start && id.localeCompare(start) < 0) return false;
            if (end && id.localeCompare(end) > 0) return false;
          }
        }

        // 2. 關鍵字篩選
        if (kw) {
          const matched = searchKeys.some(k => (item[k] || '').toLowerCase().includes(kw))
            || id.toLowerCase().includes(kw);
          if (!matched) return false;
        }

        return true;
      });

      if (limit > 0) {
        filtered = filtered.slice(0, limit);
      }

      return filtered;
    }

    async function previewBatchCards() {
      const res = await fetch('/api/readers');
      const data = await res.json();
      const settingsRes = await fetch('/api/settings');
      const sData = await settingsRes.json();
      const s = sData.data;

      const readers = filterPrintItems(data.data || [], 'id', ['name', 'class_name', 'role_name']);
      const countLabel = document.getElementById('print-count-label');
      if (countLabel) countLabel.textContent = `📋 預覽借書證：共 ${readers.length} 筆符合`;

      const area = document.getElementById('print-area');
      area.innerHTML = '';
      if (readers.length === 0) {
        area.innerHTML = '<p style="color: var(--text-muted); text-align: center; margin-top: 5rem;">查無符合所設範圍之讀者資料！請調整起訖編號或關鍵字。</p>';
        return;
      }

      const cardMargin = parseFloat(s.card_margin_mm) || 5;
      let html = `<div style="display: flex; flex-wrap: wrap; gap: ${cardMargin}mm; justify-content: flex-start; align-items: flex-start;">`;

      readers.forEach(r => {
        html += `
          <div class="print-tag-item" style="width: 86mm; height: 54mm; min-width: 86mm; min-height: 54mm; max-width: 86mm; max-height: 54mm; border: 1px solid #000; border-radius: 4px; padding: 3mm; background: white; page-break-inside: avoid; display: flex; flex-direction: column; justify-content: space-between; box-sizing: border-box; overflow: hidden;">
            <div style="font-size: 0.85rem; font-weight: bold; text-align: center; border-bottom: 1px solid #ccc; padding-bottom: 1.5mm; margin-bottom: 1.5mm; white-space: nowrap; overflow: hidden; text-overflow: ellipsis;">
              ${escapeHtml(s.library_name || '圖書管理中心')} 借書證
            </div>
            <div style="display: flex; justify-content: space-between; align-items: center; flex: 1;">
              <div>
                <div style="font-size: 1.15rem; font-weight: bold; margin-bottom: 1mm;">${escapeHtml(r.name)}</div>
                <div style="font-size: 0.8rem; color: #475569;">證號：<strong>${r.id}</strong></div>
                <div style="font-size: 0.8rem; color: #475569;">身分：${escapeHtml(r.role_name || '-')}</div>
                ${r.class_name ? `<div style="font-size: 0.8rem; color: #475569;">班級：${escapeHtml(r.class_name)}</div>` : ''}
              </div>
              <div style="width: 45px; height: 55px; border: 1px dashed #94a3b8; border-radius: 2px; display: flex; align-items: center; justify-content: center; font-size: 0.65rem; color: #64748b; background: #f8fafc;">照片處</div>
            </div>
            <div style="text-align: center; margin-top: 1mm;">
              ${generateCode39Svg(r.id, 24)}
              <div style="text-align: center; font-size: 0.72rem; letter-spacing: 2px; font-weight: bold; margin-top: 0.5mm;">*${r.id}*</div>
            </div>
          </div>
        `;
      });
      html += `</div>`;
      area.innerHTML = html;
    }

    async function previewBatchBarcodes() {
      const res = await fetch('/api/books');
      const data = await res.json();
      const settingsRes = await fetch('/api/settings');
      const sData = await settingsRes.json();
      const s = sData.data;

      const books = filterPrintItems(data.data || [], 'id', ['title', 'author', 'category', 'location']);
      const countLabel = document.getElementById('print-count-label');
      if (countLabel) countLabel.textContent = `🏷️ 預覽書籍標籤：共 ${books.length} 筆符合`;

      const area = document.getElementById('print-area');
      area.innerHTML = '';
      if (books.length === 0) {
        area.innerHTML = '<p style="color: var(--text-muted); text-align: center; margin-top: 5rem;">查無符合所設範圍之書籍資料！請調整起訖編號或關鍵字。</p>';
        return;
      }

      // 精確套用使用者設定之標籤貼紙寬度與高度 (mm)
      const wMm = parseFloat(s.barcode_width_mm) || 48;
      const hMm = parseFloat(s.barcode_height_mm) || 40;
      // 依高度動態調整條碼圖形高度
      const bcHeight = Math.max(16, Math.min(32, Math.round(hMm * 0.42)));

      // 檢查是否勾選列印背標
      const optSpine = document.getElementById('print-opt-spine');
      const showSpine = optSpine ? optSpine.checked : s.show_spine_tag;

      let html = `<div style="display: flex; flex-wrap: wrap; gap: 3mm; justify-content: flex-start; align-items: flex-start;">`;

      books.forEach(b => {
        html += `
          <div class="print-tag-item" style="width: ${wMm}mm; height: ${hMm}mm; min-width: ${wMm}mm; min-height: ${hMm}mm; max-width: ${wMm}mm; max-height: ${hMm}mm; border: 1px dashed #444; border-radius: 3px; padding: 1.5mm; background: white; page-break-inside: avoid; display: flex; box-sizing: border-box; overflow: hidden;">
            ${showSpine ? `
              <!-- 背標區域：去除固定文字 "分類號"，以大字清晰顯示分類號本體，下方保留存放位置/櫃號 -->
              <div style="width: 20mm; min-width: 20mm; border-right: 1px dashed #64748b; padding-right: 1.5mm; margin-right: 1.5mm; text-align: center; display: flex; flex-direction: column; justify-content: space-around; align-items: center; box-sizing: border-box; overflow: hidden;">
                <div style="width: 100%;">
                  <div style="font-size: 1.05rem; font-weight: 800; color: #0f172a; line-height: 1.15; word-break: break-all;">${escapeHtml(b.category || '-')}</div>
                </div>
                <div style="width: 100%; border-top: 1px dotted #cbd5e1; padding-top: 1mm;">
                  <div style="font-size: 0.58rem; color: #64748b; line-height: 1;">存放位置/櫃號</div>
                  <div style="font-size: 0.85rem; font-weight: bold; color: #1e40af; line-height: 1.15; word-break: break-all;">${escapeHtml(b.location || '-')}</div>
                </div>
              </div>
            ` : ''}
            <!-- 條碼主體區域 -->
            <div style="flex: 1; text-align: center; display: flex; flex-direction: column; justify-content: space-between; overflow: hidden; padding: 0 1mm;">
              <div style="font-size: 0.72rem; font-weight: bold; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; line-height: 1.2;" title="${escapeHtml(b.title)}">${escapeHtml(b.title)}</div>
              <div style="margin: 0.5mm 0;">${generateCode39Svg(b.id, bcHeight)}</div>
              <div style="font-size: 0.72rem; letter-spacing: 1px; font-weight: 600; line-height: 1;">*${b.id}*</div>
              <div style="font-size: 0.62rem; color: #475569; white-space: nowrap; overflow: hidden; text-overflow: ellipsis;">${escapeHtml(s.library_name || '')}</div>
            </div>
          </div>
        `;
      });
      html += `</div>`;
      area.innerHTML = html;
    }

    // === 8. 統計儀表板 ===
    async function loadStats() {
      const res = await fetch('/api/stats');
      const data = await res.json();
      if (data.success && data.data) {
        const s = data.data;
        document.getElementById('stat-total-books').textContent = s.total_books;
        document.getElementById('stat-available-books').textContent = s.available_books;
        document.getElementById('stat-borrowed-books').textContent = s.borrowed_books;
        document.getElementById('stat-overdue-loans').textContent = s.overdue_loans;

        // 熱門榜
        const topList = document.getElementById('top-books-list');
        topList.innerHTML = '';
        s.top_borrowed_books.forEach((b, i) => {
          topList.innerHTML += `
            <div style="display: flex; justify-content: space-between; align-items: center; padding: 0.5rem 0; border-bottom: 1px solid var(--border);">
              <span><strong>#${i + 1}</strong> ${escapeHtml(b.title)}</span>
              <span class="badge badge-info">${b.borrow_count} 次借閱</span>
            </div>
          `;
        });

        // 分類長條圖
        const catChart = document.getElementById('category-chart');
        catChart.innerHTML = '';
        s.category_distribution.forEach(c => {
          const max = s.total_books || 1;
          const pct = Math.round((c.count / max) * 100);
          catChart.innerHTML += `
            <div style="margin-bottom: 0.75rem;">
              <div style="display: flex; justify-content: space-between; font-size: 0.85rem; margin-bottom: 0.25rem;">
                <span>${escapeHtml(c.category)}</span>
                <span>${c.count} 本 (${pct}%)</span>
              </div>
              <div style="background: #e2e8f0; height: 10px; border-radius: 999px; overflow: hidden;">
                <div style="background: var(--primary); width: ${pct}%; height: 100%;"></div>
              </div>
            </div>
          `;
        });

        // 最近歸還有逾期之歷史紀錄
        const overdueTableBody = document.querySelector('#recent-overdue-returns-table tbody');
        if (overdueTableBody) {
          overdueTableBody.innerHTML = '';
          const list = s.recent_overdue_returns || [];
          if (list.length === 0) {
            overdueTableBody.innerHTML = '<tr><td colspan="6" style="text-align: center; color: var(--text-muted); padding: 1.5rem;">目前尚無已歸還逾期之歷史紀錄，讀者皆按時歸還！🎉</td></tr>';
          } else {
            list.forEach(item => {
              const tr = document.createElement('tr');
              tr.innerHTML = `
                <td><strong>${escapeHtml(item.reader_name)}</strong> <span style="font-size: 0.8rem; color: var(--text-muted);">(證號: ${item.reader_id})</span></td>
                <td><strong>${escapeHtml(item.book_title)}</strong> <span style="font-size: 0.8rem; color: var(--text-muted);">(條碼: ${item.book_id})</span></td>
                <td>${item.borrow_date}</td>
                <td>${item.due_date}</td>
                <td style="color: #475569;">${item.return_date}</td>
                <td><span class="badge badge-danger" style="font-size: 0.82rem; font-weight: bold;">逾期 ${item.overdue_days} 天</span></td>
              `;
              overdueTableBody.appendChild(tr);
            });
          }
        }
      }
    }

    // === 9. 系統設定 ===
    async function loadSettings() {
      const res = await fetch('/api/settings');
      const data = await res.json();
      if (data.success && data.data) {
        const s = data.data;
        document.getElementById('set-lib-name').value = s.library_name;
        document.getElementById('header-library-name').textContent = s.library_name;
        document.getElementById('set-lib-phone').value = s.library_phone;
        document.getElementById('set-lib-address').value = s.library_address;
        document.getElementById('set-lib-rules').value = s.borrow_rules;
        document.getElementById('set-barcode-w').value = s.barcode_width_mm;
        document.getElementById('set-barcode-h').value = s.barcode_height_mm;
        document.getElementById('set-show-spine').checked = s.show_spine_tag;
      }
    }

    async function saveSettings() {
      const settings = {
        library_name: document.getElementById('set-lib-name').value.trim(),
        library_phone: document.getElementById('set-lib-phone').value.trim(),
        library_address: document.getElementById('set-lib-address').value.trim(),
        borrow_rules: document.getElementById('set-lib-rules').value.trim(),
        barcode_width_mm: parseFloat(document.getElementById('set-barcode-w').value) || 48,
        barcode_height_mm: parseFloat(document.getElementById('set-barcode-h').value) || 40,
        show_spine_tag: document.getElementById('set-show-spine').checked,
        card_margin_mm: 5,
        requests_per_minute: 20,
        jitter_ms: 1000,
      };
      const res = await fetch('/api/settings', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(settings)
      });
      const data = await res.json();
      alert(data.message);
      if (data.success) {
        document.getElementById('header-library-name').textContent = settings.library_name;
      }
    }

    async function triggerLegacyImport() {
      if (!confirm('確定要重新讀取 data/ 目錄下的 DBF 舊資料庫嗎？')) return;
      const res = await fetch('/api/system/import-legacy', { method: 'POST' });
      const data = await res.json();
      alert(data.message);
      loadStats();
    }

    // 初始化載入
    window.addEventListener('DOMContentLoaded', () => {
      loadSettings();
      loadRoles();
      document.getElementById('borrow-reader-id').focus();
    });
  </script>
</body>
</html>
"#;
