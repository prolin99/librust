pub const HTML_PAGE: &str = r##"<!DOCTYPE html>
<html lang="zh-TW">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>現代化圖書管理借還系統</title>
  <link rel="icon" type="image/x-icon" href="/favicon.ico">
  <link rel="icon" type="image/png" href="/app.png">
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

  <!-- 登入驗證覆蓋層 (未登入時阻擋操作) -->
  <div id="login-overlay" style="position: fixed; inset: 0; background: rgba(15, 23, 42, 0.92); backdrop-filter: blur(4px); display: flex; align-items: center; justify-content: center; z-index: 99999;">
    <div style="background: white; border-radius: 0.75rem; width: 100%; max-width: 420px; padding: 2.25rem 2rem; box-shadow: 0 25px 50px -12px rgba(0,0,0,0.5); border: 1px solid var(--border);">
      <div style="text-align: center; margin-bottom: 1.5rem;">
        <div style="font-size: 3rem; margin-bottom: 0.5rem;">📚</div>
        <h2 style="font-size: 1.35rem; color: var(--text); font-weight: bold;" id="login-app-title">圖書管理借還系統</h2>
        <p style="color: var(--text-muted); font-size: 0.88rem; margin-top: 0.25rem;">系統身分驗證，請登入後使用</p>
      </div>
      
      <div id="login-error-msg" style="display: none; background: #fee2e2; color: #b91c1c; padding: 0.6rem 0.75rem; border-radius: 0.375rem; font-size: 0.85rem; margin-bottom: 1rem; border: 1px solid #f87171; font-weight: 500;"></div>

      <div style="margin-bottom: 1rem;">
        <label style="display: block; font-weight: 600; font-size: 0.9rem; margin-bottom: 0.35rem; color: #334155;">帳號</label>
        <input type="text" id="login-username" value="admin" placeholder="請輸入帳號" autocomplete="username" style="font-size: 1rem; padding: 0.65rem 0.75rem;" onkeydown="if(event.key==='Enter') document.getElementById('login-password').focus()">
      </div>

      <div style="margin-bottom: 1.25rem;">
        <label style="display: block; font-weight: 600; font-size: 0.9rem; margin-bottom: 0.35rem; color: #334155;">密碼</label>
        <input type="password" id="login-password" placeholder="請輸入密碼" autocomplete="current-password" style="font-size: 1rem; padding: 0.65rem 0.75rem;" onkeydown="if(event.key==='Enter') doLogin()">
      </div>

      <button class="btn btn-primary" style="width: 100%; padding: 0.75rem; font-size: 1rem; font-weight: 600;" onclick="doLogin()">登入系統</button>

      <div style="margin-top: 1.25rem; padding: 0.75rem; background: #f8fafc; border-radius: 0.375rem; font-size: 0.82rem; color: #475569; text-align: left; border: 1px dashed #cbd5e1; line-height: 1.5;">
        💡 <strong>提示</strong>：第一次進入帳號、密碼預設均為 <code style="background: #e2e8f0; padding: 0.15rem 0.35rem; border-radius: 0.2rem; font-weight: bold; color: #1e293b;">admin</code>。<br>
        進入系統後可至「⚙️ 系統設定」更改密碼，密碼以安全加密格式保存於資料庫中。
      </div>

      <div style="margin-top: 0.85rem; text-align: center;">
        <button type="button" class="btn btn-outline btn-sm" style="color: #ef4444; border-color: #fca5a5; font-size: 0.82rem; padding: 0.35rem 0.75rem;" onclick="handleShutdown()">🛑 關閉系統服務</button>
      </div>
    </div>
  </div>

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
      <button onclick="switchTab('print')">🖨️ 證卡標籤列印</button>
      <button onclick="switchTab('stats')">📊 統計儀表板</button>
      <button onclick="switchTab('settings')">⚙️ 系統設定</button>
    </nav>
    <div id="user-info-bar" style="display: flex; align-items: center; gap: 0.75rem;">
      <span id="current-user-display" style="font-size: 0.9rem; color: #cbd5e1; font-weight: 500;">👤 admin</span>
      <button class="btn btn-outline btn-sm" style="color: white; border-color: #475569;" onclick="handleLogout()">🚪 登出</button>
      <button class="btn btn-sm" style="color: #ffffff; background: #dc2626; border: 1px solid #b91c1c; font-weight: 600;" onclick="handleShutdown()" title="安全關閉圖書管理系統背景伺服器">🛑 關閉系統</button>
    </div>
  </header>

  <!-- 定期備份提醒橫幅 -->
  <div id="backup-reminder-banner" class="no-print" style="display: none; background: #fef3c7; border-bottom: 1px solid #f59e0b; color: #92400e; padding: 0.65rem 1.5rem; font-size: 0.9rem;">
    <div style="max-width: 1400px; margin: 0 auto; display: flex; justify-content: space-between; align-items: center; gap: 0.75rem; flex-wrap: wrap;">
      <span id="backup-reminder-text" style="font-weight: 600;">⚠️ 提醒：建議定期備份資料庫！</span>
      <div style="display: flex; gap: 0.5rem; align-items: center;">
        <button type="button" id="btn-banner-backup-now" class="btn btn-sm" style="background: #d97706; color: white;" onclick="createDatabaseBackup(true)">💾 立即備份</button>
        <button type="button" class="btn btn-outline btn-sm" style="background: white; color: #92400e; border-color: #f59e0b;" onclick="dismissBackupReminder()">稍後提醒</button>
      </div>
    </div>
  </div>

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
            <button class="btn btn-success" onclick="exportActiveLoans()">📥 匯出 Excel (CSV)</button>
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
              <option value="200">200 筆</option>
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
          <div style="display: flex; gap: 0.5rem; flex: 1; min-width: 320px; max-width: 680px; flex-wrap: wrap;">
            <input type="text" id="books-search" placeholder="搜尋書名、條碼、作者、ISBN、分類..." style="flex: 1; min-width: 180px;" onkeydown="if(event.key==='Enter') loadBooks()">
            <select id="books-status-filter" style="width: 125px;" onchange="loadBooks()">
              <option value="">全部狀態</option>
              <option value="available">在館可借</option>
              <option value="borrowed">借出中</option>
            </select>
            <select id="books-category-filter" style="width: 165px;" onchange="loadBooks()">
              <option value="">全部分類</option>
            </select>
            <button class="btn btn-primary" onclick="loadBooks()">搜尋</button>
          </div>
          <div style="display: flex; gap: 0.5rem; flex-wrap: wrap;">
            <button class="btn btn-outline" onclick="openCategoryManagerModal()">🏷️ 管理大分類</button>
            <button class="btn btn-outline" onclick="document.getElementById('import-books-input').click()">📥 匯入 Excel/CSV</button>
            <input type="file" id="import-books-input" style="display: none;" accept=".xlsx,.xls,.csv" onchange="uploadBooksFile(this)">
            <a href="/api/templates/books.csv" class="btn btn-outline" download>📄 下載範本</a>
            <a href="/api/export/books.csv" class="btn btn-outline" download>📥 匯出 Excel/CSV</a>
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
              <option value="200">200 筆</option>
            </select>
            <span id="books-page-summary" style="color: var(--text-muted); margin-left: 0.5rem;">載入中...</span>
          </div>
          <div style="display: flex; align-items: center; gap: 0.35rem;">
            <button class="btn btn-outline btn-sm" id="btn-books-first" onclick="gotoBooksPage(1)">⏮ 首頁</button>
            <button class="btn btn-outline btn-sm" id="btn-books-prev" onclick="prevBooksPage()">◀ 上一頁</button>
            <select id="books-page-indicator" aria-label="選擇書籍頁次" style="width: auto; min-width: 110px; padding: 0.25rem 0.5rem; font-size: 0.85rem; font-weight: 600;" onchange="gotoBooksPage(parseInt(this.value, 10))">
              <option value="1">第 1 / 1 頁</option>
            </select>
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
            <select id="books-page-indicator-bottom" aria-label="選擇書籍頁次" style="width: auto; min-width: 110px; padding: 0.25rem 0.5rem; font-size: 0.85rem; font-weight: 600;" onchange="gotoBooksPage(parseInt(this.value, 10))">
              <option value="1">第 1 / 1 頁</option>
            </select>
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
            <input type="text" id="readers-search" placeholder="搜尋讀者姓名、證號、班級、備註..." onkeydown="if(event.key==='Enter') loadReaders()">
            <button class="btn btn-primary" onclick="loadReaders()">搜尋</button>
          </div>
          <div style="display: flex; gap: 0.5rem;">
            <button class="btn btn-outline" onclick="document.getElementById('import-readers-input').click()">📥 匯入 Excel/CSV</button>
            <input type="file" id="import-readers-input" style="display: none;" accept=".xlsx,.xls,.csv" onchange="uploadReadersFile(this)">
            <a href="/api/templates/readers.csv" class="btn btn-outline" download>📄 下載範本</a>
            <a href="/api/export/readers.csv" class="btn btn-outline" download>📥 匯出 Excel/CSV</a>
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
              <option value="200">200 筆</option>
            </select>
            <span id="readers-page-summary" style="color: var(--text-muted); margin-left: 0.5rem;">載入中...</span>
          </div>
          <div style="display: flex; align-items: center; gap: 0.35rem;">
            <button class="btn btn-outline btn-sm" id="btn-readers-first" onclick="gotoReadersPage(1)">⏮ 首頁</button>
            <button class="btn btn-outline btn-sm" id="btn-readers-prev" onclick="prevReadersPage()">◀ 上一頁</button>
            <select id="readers-page-indicator" aria-label="選擇讀者頁次" style="width: auto; min-width: 110px; padding: 0.25rem 0.5rem; font-size: 0.85rem; font-weight: 600;" onchange="gotoReadersPage(parseInt(this.value, 10))">
              <option value="1">第 1 / 1 頁</option>
            </select>
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
                <th>備註</th>
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
            <select id="readers-page-indicator-bottom" aria-label="選擇讀者頁次" style="width: auto; min-width: 110px; padding: 0.25rem 0.5rem; font-size: 0.85rem; font-weight: 600;" onchange="gotoReadersPage(parseInt(this.value, 10))">
              <option value="1">第 1 / 1 頁</option>
            </select>
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


    <!-- 7. 證卡標籤批次列印 -->
    <section id="tab-print" class="tab-content">
      <div class="card">
        <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 1rem; flex-wrap: wrap; gap: 0.5rem;">
          <h2>🖨️ 借書證與書籍標籤批次列印</h2>
          <div style="display: flex; gap: 0.5rem;">
            <button class="btn btn-outline" onclick="previewBatchCards()">🪪 預覽借書證</button>
            <button class="btn btn-outline" onclick="previewBatchBarcodes()">🏷️ 預覽書籍標籤</button>
            <button class="btn btn-primary" onclick="doDirectPrint()">🖨️ 直接列印</button>
          </div>
        </div>

        <!-- 列印範圍自訂設定列 -->
        <div style="background: #f8fafc; border: 1px solid var(--border); border-radius: 0.375rem; padding: 1rem; margin-bottom: 1.25rem;">
          <div style="font-weight: 600; font-size: 0.9rem; margin-bottom: 0.5rem; color: #334155;">🎯 列印範圍設定（可指定起訖條碼/證號，或留空列印全部）：</div>
          <div style="display: grid; grid-template-columns: repeat(auto-fit, minmax(180px, 1fr)); gap: 0.75rem; align-items: flex-end;">
            <div>
              <label style="font-size: 0.8rem; color: var(--text-muted);">起始編號 (條碼/證號)：</label>
              <input type="text" id="print-range-start" placeholder="例：B000001 或 00001" style="margin-top: 0.25rem;" oninput="onPrintOptionDebounced()">
            </div>
            <div>
              <label style="font-size: 0.8rem; color: var(--text-muted);">結束編號 (條碼/證號)：</label>
              <input type="text" id="print-range-end" placeholder="例：B000050 或 00050" style="margin-top: 0.25rem;" oninput="onPrintOptionDebounced()">
            </div>
            <div>
              <label style="font-size: 0.8rem; color: var(--text-muted);">關鍵字 / 分類 / 身分篩選：</label>
              <input type="text" id="print-range-filter" placeholder="如書名、作者、班級..." style="margin-top: 0.25rem;" oninput="onPrintOptionDebounced()">
            </div>
            <div>
              <label style="font-size: 0.8rem; color: var(--text-muted);">最多筆數限制：</label>
              <select id="print-range-limit" style="margin-top: 0.25rem;" onchange="triggerInstantPreview()">
                <option value="0">全部符合項目</option>
                <option value="20">前 20 筆</option>
                <option value="50" selected>前 50 筆</option>
                <option value="100">前 100 筆</option>
                <option value="200">前 200 筆</option>
              </select>
            </div>
            <div style="display: flex; align-items: center; padding-bottom: 0.5rem;">
              <label style="font-size: 0.85rem; font-weight: 600; cursor: pointer; display: flex; align-items: center; gap: 0.35rem; color: #1e293b;">
                <input type="checkbox" id="print-opt-spine" checked style="width: 1rem; height: 1rem; cursor: pointer;" onchange="onSpineCheckboxChange()"> 書籍標籤包含背標
              </label>
            </div>
          </div>
          <div id="print-status-bar" style="margin-top: 0.75rem; font-size: 0.85rem; color: #475569; display: flex; justify-content: space-between; align-items: center; flex-wrap: wrap; gap: 0.5rem;">
            <span>💡 提示：設定起訖編號後，點選上方「預覽借書證」或「預覽書籍標籤」即可套用篩選。</span>
            <span id="print-count-label" style="font-weight: 600;"></span>
          </div>

          <!-- 條碼貼紙 / 證卡尺寸設定與即時預覽 -->
          <div style="margin-top: 0.85rem; padding-top: 0.85rem; border-top: 1px dashed #cbd5e1; display: flex; align-items: center; flex-wrap: wrap; gap: 0.85rem;">
            <div id="print-dim-title" style="font-weight: 600; font-size: 0.85rem; color: #334155;">
              📏 列印尺寸設定：
            </div>
            <div style="display: flex; align-items: center; gap: 0.4rem;">
              <label style="font-size: 0.82rem; color: var(--text-muted); font-weight: 600; white-space: nowrap;">寬度 (mm)：</label>
              <input type="number" id="set-barcode-w" value="48" style="width: 80px; margin: 0; padding: 0.35rem 0.5rem;" oninput="onBarcodeDimensionChange()" onchange="onBarcodeDimensionChange()">
            </div>
            <div style="display: flex; align-items: center; gap: 0.4rem;">
              <label style="font-size: 0.82rem; color: var(--text-muted); font-weight: 600; white-space: nowrap;">高度 (mm)：</label>
              <input type="number" id="set-barcode-h" value="40" style="width: 80px; margin: 0; padding: 0.35rem 0.5rem;" oninput="onBarcodeDimensionChange()" onchange="onBarcodeDimensionChange()">
            </div>
            <button type="button" class="btn btn-outline" style="padding: 0.3rem 0.75rem; font-size: 0.82rem;" onclick="triggerInstantPreview()">🔄 立即更新預覽</button>
            <div id="barcode-dim-reminder" style="display: none; padding: 0.35rem 0.75rem; border-radius: 0.375rem; font-size: 0.82rem; font-weight: 600;"></div>
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

      <!-- 期間借閱統計分析：月份統計圖 + 個人/書籍排行 + 匯出 -->
      <div class="card" style="margin-bottom: 1.5rem;">
        <div style="display: flex; justify-content: space-between; align-items: center; flex-wrap: wrap; gap: 0.75rem; margin-bottom: 1rem;">
          <div>
            <h3 style="margin: 0;">📅 期間借閱統計分析（月份人次／冊數與排行榜）</h3>
            <p style="font-size: 0.82rem; color: var(--text-muted); margin-top: 0.25rem;">
              依指定時間範圍統計每月借閱人次（同讀者同日計 1 人次）與借閱冊數，並列出個人借閱與書籍被借排行。
            </p>
          </div>
          <div style="display: flex; gap: 0.4rem; flex-wrap: wrap;">
            <button type="button" class="btn btn-outline btn-sm" onclick="setStatsPreset('6m')">近 6 個月</button>
            <button type="button" class="btn btn-outline btn-sm" onclick="setStatsPreset('1y')">近 1 年</button>
            <button type="button" class="btn btn-outline btn-sm" onclick="setStatsPreset('year')">今年度</button>
            <button type="button" class="btn btn-outline btn-sm" onclick="setStatsPreset('all')">全部期間</button>
          </div>
        </div>

        <div style="display: flex; align-items: flex-end; gap: 0.75rem; flex-wrap: wrap; background: #f8fafc; padding: 0.85rem 1rem; border-radius: 0.5rem; border: 1px solid var(--border); margin-bottom: 1rem;">
          <div>
            <label style="display: block; font-size: 0.8rem; font-weight: 600; color: var(--text-muted); margin-bottom: 0.25rem;">開始日期</label>
            <input type="date" id="stats-start-date" style="width: 155px; padding: 0.4rem 0.6rem;">
          </div>
          <div>
            <label style="display: block; font-size: 0.8rem; font-weight: 600; color: var(--text-muted); margin-bottom: 0.25rem;">結束日期</label>
            <input type="date" id="stats-end-date" style="width: 155px; padding: 0.4rem 0.6rem;">
          </div>
          <div>
            <label style="display: block; font-size: 0.8rem; font-weight: 600; color: var(--text-muted); margin-bottom: 0.25rem;">排行筆數 <span style="font-weight: normal;">(0=全部)</span></label>
            <input type="number" id="stats-limit" value="20" min="0" max="10000" style="width: 100px; padding: 0.4rem 0.6rem;">
          </div>
          <button type="button" class="btn btn-primary" onclick="loadRangeStats()">🔍 查詢統計</button>
          <div style="margin-left: auto; display: flex; gap: 0.5rem; flex-wrap: wrap;">
            <button type="button" class="btn btn-outline btn-sm" style="color: var(--success); border-color: var(--success);" onclick="exportRangeStatsCsv('monthly')">📥 匯出月份統計 (Excel / CSV)</button>
          </div>
        </div>

        <!-- 期間總覽指標 -->
        <div style="display: flex; gap: 1rem; flex-wrap: wrap; margin-bottom: 1rem;">
          <div style="flex: 1; min-width: 180px; background: #ecfdf5; border: 1px solid #a7f3d0; border-radius: 0.5rem; padding: 0.75rem 1rem; display: flex; justify-content: space-between; align-items: center;">
            <span style="font-size: 0.88rem; color: #065f46; font-weight: 600;">👥 期間總借閱人次</span>
            <span id="range-total-visits" style="font-size: 1.5rem; font-weight: bold; color: #059669;">-</span>
          </div>
          <div style="flex: 1; min-width: 180px; background: #eff6ff; border: 1px solid #bfdbfe; border-radius: 0.5rem; padding: 0.75rem 1rem; display: flex; justify-content: space-between; align-items: center;">
            <span style="font-size: 0.88rem; color: #1e40af; font-weight: 600;">📖 期間總借閱冊數</span>
            <span id="range-total-books" style="font-size: 1.5rem; font-weight: bold; color: #2563eb;">-</span>
          </div>
          <div style="flex: 1; min-width: 180px; background: #f8fafc; border: 1px solid var(--border); border-radius: 0.5rem; padding: 0.75rem 1rem; display: flex; justify-content: space-between; align-items: center;">
            <span style="font-size: 0.88rem; color: #475569; font-weight: 600;">📆 統計月份數</span>
            <span id="range-month-count" style="font-size: 1.5rem; font-weight: bold; color: #334155;">-</span>
          </div>
        </div>

        <!-- 月份借閱統計圖 (SVG) -->
        <div style="border: 1px solid var(--border); border-radius: 0.5rem; padding: 1rem; background: #ffffff; margin-bottom: 1rem;">
          <div style="display: flex; justify-content: space-between; align-items: center; flex-wrap: wrap; gap: 0.5rem; margin-bottom: 0.75rem;">
            <strong style="font-size: 0.95rem;">📊 每月借閱人次與冊數統計圖</strong>
            <div style="display: flex; gap: 1rem; font-size: 0.82rem; align-items: center;">
              <span style="display: inline-flex; align-items: center; gap: 0.35rem;"><span style="width: 12px; height: 12px; background: #10b981; border-radius: 2px; display: inline-block;"></span>借閱人次</span>
              <span style="display: inline-flex; align-items: center; gap: 0.35rem;"><span style="width: 12px; height: 12px; background: #2563eb; border-radius: 2px; display: inline-block;"></span>借閱冊數</span>
            </div>
          </div>
          <div id="monthly-borrow-chart" style="overflow-x: auto; min-height: 240px;">
            <p style="text-align: center; color: var(--text-muted); padding: 3rem 0;">載入統計圖中...</p>
          </div>
        </div>

        <!-- 月份統計明細表（可折疊） -->
        <details style="margin-bottom: 1.25rem;">
          <summary style="cursor: pointer; font-size: 0.88rem; font-weight: 600; color: var(--primary); padding: 0.35rem 0;">📋 展開／收合月份借閱明細表</summary>
          <div class="table-container" style="margin-top: 0.5rem; max-height: 260px; overflow-y: auto;">
            <table id="monthly-stats-table">
              <thead>
                <tr>
                  <th>月份</th>
                  <th>借閱人次</th>
                  <th>借閱冊數</th>
                </tr>
              </thead>
              <tbody>
                <tr><td colspan="3" style="text-align: center; color: var(--text-muted);">載入中...</td></tr>
              </tbody>
            </table>
          </div>
        </details>

        <!-- 期間內個人借閱冊數排行 & 書籍被借次數排行 -->
        <div class="grid grid-2">
          <div style="border: 1px solid var(--border); border-radius: 0.5rem; padding: 1rem; background: #ffffff;">
            <div style="display: flex; justify-content: space-between; align-items: center; flex-wrap: wrap; gap: 0.5rem; margin-bottom: 0.75rem;">
              <h4 style="margin: 0;">👤 個人借閱冊數排行 <span id="reader-rank-badge" class="badge badge-info">前 20 筆</span></h4>
              <button type="button" class="btn btn-outline btn-sm" style="color: var(--primary); border-color: var(--primary);" onclick="exportRangeStatsCsv('readers')">📥 匯出 Excel / CSV</button>
            </div>
            <div class="table-container" style="max-height: 360px; overflow-y: auto;">
              <table id="reader-rank-table">
                <thead>
                  <tr>
                    <th style="width: 55px;">排名</th>
                    <th>讀者證號 / 姓名</th>
                    <th>身分 / 班級</th>
                    <th style="text-align: right;">人次</th>
                    <th style="text-align: right;">冊數</th>
                  </tr>
                </thead>
                <tbody>
                  <tr><td colspan="5" style="text-align: center; color: var(--text-muted); padding: 1.5rem;">載入中...</td></tr>
                </tbody>
              </table>
            </div>
          </div>

          <div style="border: 1px solid var(--border); border-radius: 0.5rem; padding: 1rem; background: #ffffff;">
            <div style="display: flex; justify-content: space-between; align-items: center; flex-wrap: wrap; gap: 0.5rem; margin-bottom: 0.75rem;">
              <h4 style="margin: 0;">📚 書籍被借次數排行 <span id="book-rank-badge" class="badge badge-info">前 20 筆</span></h4>
              <button type="button" class="btn btn-outline btn-sm" style="color: var(--primary); border-color: var(--primary);" onclick="exportRangeStatsCsv('books')">📥 匯出 Excel / CSV</button>
            </div>
            <div class="table-container" style="max-height: 360px; overflow-y: auto;">
              <table id="book-rank-table">
                <thead>
                  <tr>
                    <th style="width: 55px;">排名</th>
                    <th>書籍條碼 / 書名</th>
                    <th>作者 / 分類</th>
                    <th style="text-align: right;">被借次數</th>
                  </tr>
                </thead>
                <tbody>
                  <tr><td colspan="4" style="text-align: center; color: var(--text-muted); padding: 1.5rem;">載入中...</td></tr>
                </tbody>
              </table>
            </div>
          </div>
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
        <h2 style="margin-bottom: 1rem;">⚙️ 圖書館基本資料設定</h2>
        <div style="margin-bottom: 0.75rem;">
          <label style="font-weight: 600;">學校 / 機構名稱 (印在借書證與書標籤)</label>
          <input type="text" id="set-lib-name" style="margin-top: 0.25rem;">
        </div>
        <div style="margin-bottom: 0.75rem;">
          <label style="font-weight: 600;">電話</label>
          <input type="text" id="set-lib-phone" style="margin-top: 0.25rem;">
        </div>
        <div style="margin-bottom: 1.25rem;">
          <label style="font-weight: 600;">地址</label>
          <input type="text" id="set-lib-address" style="margin-top: 0.25rem;">
        </div>
        <button class="btn btn-primary" onclick="saveSettings()">儲存系統設定</button>

        <hr style="margin: 2rem 0; border: none; border-top: 1px solid var(--border);">

        <h3 style="color: var(--primary); margin-bottom: 0.5rem;">📊 系統資料匯出 (Excel / CSV)</h3>
        <p style="color: var(--text-muted); font-size: 0.85rem; margin-bottom: 1rem;">
          提供將讀者資料與書籍資料完整匯出為 Excel 相容之 CSV 試算表（含 UTF-8 BOM），可直接以 Microsoft Excel、WPS、Google 試算表開啟，適合全館資料備份、報表製作與統計檢視。
        </p>
        <div style="display: flex; gap: 0.75rem; flex-wrap: wrap;">
          <a href="/api/export/readers.csv" class="btn btn-outline" style="color: var(--primary); border-color: var(--primary); text-decoration: none;" download>
            📥 匯出讀者資料 (Excel / CSV)
          </a>
          <a href="/api/export/books.csv" class="btn btn-outline" style="color: var(--primary); border-color: var(--primary); text-decoration: none;" download>
            📥 匯出書籍資料 (Excel / CSV)
          </a>
        </div>

        <hr style="margin: 2rem 0; border: none; border-top: 1px solid var(--border);">

        <h3 style="color: var(--primary); margin-bottom: 0.5rem;">🔐 變更管理員登入密碼</h3>
        <p style="color: var(--text-muted); font-size: 0.85rem; margin-bottom: 1rem;">
          密碼記錄經不可逆之安全雜湊（bcrypt 加密）存入資料庫，絕非明碼儲存，保障系統安全。
        </p>
        <div style="margin-bottom: 0.75rem;">
          <label style="font-weight: 600;">目前密碼</label>
          <input type="password" id="change-pwd-current" placeholder="請輸入目前密碼" style="margin-top: 0.25rem;">
        </div>
        <div style="margin-bottom: 0.75rem;">
          <label style="font-weight: 600;">新密碼</label>
          <input type="password" id="change-pwd-new" placeholder="請輸入新密碼" style="margin-top: 0.25rem;">
        </div>
        <div style="margin-bottom: 1rem;">
          <label style="font-weight: 600;">確認新密碼</label>
          <input type="password" id="change-pwd-confirm" placeholder="請再次輸入新密碼" style="margin-top: 0.25rem;">
        </div>
        <div id="change-pwd-msg" style="display: none; padding: 0.6rem; border-radius: 0.375rem; font-size: 0.85rem; margin-bottom: 1rem;"></div>
        <button class="btn btn-warning" onclick="handleChangePassword()">變更密碼</button>

        <hr style="margin: 2rem 0; border: none; border-top: 1px solid var(--border);">

        <h3 style="color: var(--primary); margin-bottom: 0.5rem;">💾 資料庫備份與還原</h3>
        <p style="color: var(--text-muted); font-size: 0.85rem; margin-bottom: 1rem;">
          將 <code>library.db</code> 完整備份至指定目錄（系統會自動保留最新 <strong>10 份</strong>常規備份，超過自動刪除最舊者，也可手動刪除），並支援從舊備份還原資料。
        </p>

        <div style="margin-bottom: 0.75rem;">
          <label style="font-weight: 600; display: block; margin-bottom: 0.25rem;">備份儲存目錄</label>
          <div style="display: flex; gap: 0.5rem; flex-wrap: wrap;">
            <input type="text" id="backup-dir-input" placeholder="例如 D:\圖書館備份 或 E:\backup" style="flex: 1; min-width: 240px;">
            <button type="button" class="btn btn-outline" onclick="pickBackupDir()">📂 選擇資料夾</button>
          </div>
        </div>

        <div style="display: flex; gap: 1rem; align-items: flex-end; flex-wrap: wrap; margin-bottom: 0.85rem;">
          <div style="min-width: 220px;">
            <label style="font-weight: 600; display: block; margin-bottom: 0.25rem;">定期備份提醒週期</label>
            <select id="backup-reminder-cycle">
              <option value="off">關閉提醒</option>
              <option value="weekly" selected>每週提醒（超過 7 天）</option>
              <option value="monthly">每月提醒（超過 30 天）</option>
            </select>
          </div>
          <div id="backup-last-info" style="font-size: 0.85rem; color: var(--text-muted); padding-bottom: 0.45rem;">
            上次備份時間：載入中...
          </div>
        </div>

        <div style="display: flex; gap: 0.5rem; flex-wrap: wrap; margin-bottom: 1rem;">
          <button type="button" class="btn btn-outline" onclick="saveBackupConfig()">💾 儲存備份設定</button>
          <button type="button" id="btn-create-backup" class="btn btn-primary" onclick="createDatabaseBackup(false)">⚡ 立即備份資料庫</button>
          <a href="/api/system/backup/download-current" class="btn btn-outline" style="text-decoration: none;" download>⬇️ 下載目前資料庫複本</a>
        </div>

        <div id="backup-action-status" style="display: none; margin-bottom: 1rem; padding: 0.7rem 1rem; border-radius: 0.375rem; font-size: 0.88rem;"></div>

        <div style="background: #eff6ff; border: 1px solid #bfdbfe; border-radius: 0.375rem; padding: 0.75rem 1rem; font-size: 0.84rem; color: #1e3a8a; margin-bottom: 1rem;">
          ℹ️ <strong>還原注意事項：</strong>還原舊備份會取回該時間點的書籍、讀者、借閱紀錄與身分別，但會<strong>保留目前最新的管理員帳號密碼（不會還原舊帳號密碼）</strong>。還原前系統會自動為現有資料庫建立一份快照（<code>library_before_restore_*.db</code>），且若還原舊版本資料庫會自動升級資料表格式。
        </div>

        <div style="display: flex; justify-content: space-between; align-items: center; flex-wrap: wrap; gap: 0.5rem; margin-bottom: 0.5rem;">
          <strong style="font-size: 0.92rem;">📁 備份目錄現有檔案（由新到舊）</strong>
          <div style="display: flex; gap: 0.5rem; align-items: center;">
            <input type="file" id="restore-db-file" accept=".db,.sqlite,.sqlite3" style="display: none;" onchange="uploadRestoreDbFile(this)">
            <button type="button" class="btn btn-outline btn-sm" onclick="document.getElementById('restore-db-file').click()">📂 選擇其他位置 .db 檔還原</button>
            <button type="button" class="btn btn-outline btn-sm" onclick="loadBackupStatus()">🔄 重新整理清單</button>
          </div>
        </div>

        <div class="table-container" style="max-height: 280px; overflow-y: auto;">
          <table id="backup-files-table">
            <thead>
              <tr>
                <th>備份檔名</th>
                <th>建立時間</th>
                <th>大小</th>
                <th>操作</th>
              </tr>
            </thead>
            <tbody>
              <tr><td colspan="4" style="text-align: center; color: var(--text-muted); padding: 1.25rem;">載入中...</td></tr>
            </tbody>
          </table>
        </div>

        <hr style="margin: 2rem 0; border: none; border-top: 1px solid var(--border);">

        <h3 style="margin-bottom: 0.5rem;">🚀 系統版本與線上自動更新</h3>
        <p id="settings-update-info" style="color: var(--text-muted); font-size: 0.85rem; margin-bottom: 0.85rem;">
          目前版本：<code>v0.12</code>（正在檢查 GitHub Releases 是否有新版...）
        </p>
        <div style="display: flex; gap: 0.5rem; flex-wrap: wrap; align-items: center;">
          <button type="button" class="btn btn-outline" onclick="checkForUpdates(true)">🔍 檢查新版</button>
          <button type="button" id="btn-apply-update-settings" class="btn btn-primary" style="display: none;" onclick="applySystemUpdate()">⬇️ 一鍵下載並更新</button>
        </div>
        <div id="apply-update-status" style="display: none; margin-top: 0.75rem; padding: 0.75rem 1rem; border-radius: 0.375rem; font-size: 0.9rem;"></div>

        <hr style="margin: 2rem 0; border: none; border-top: 1px solid var(--border);">

        <h3 style="color: var(--danger); margin-bottom: 0.5rem;">🔄 從舊系統重新移轉資料</h3>
        <p style="color: var(--text-muted); font-size: 0.85rem; margin-bottom: 0.75rem;">
          讀取本機 <code>data/</code> 目錄下的 DBF01、Dbf02、Dbf03、Liborrow 等舊版 DBF 檔案並匯入。<br>
          ⏳ <strong>提醒：</strong>若舊版 DBF 檔案（含書籍與歷史借閱紀錄）資料量較大，讀取與移轉可能需要數十秒至數分鐘，執行期間請耐心等待，切勿關閉視窗或重新整理頁面。
        </p>
        <label style="display: flex; align-items: center; gap: 0.5rem; margin-bottom: 0.85rem; font-size: 0.9rem; cursor: pointer;">
          <input type="checkbox" id="legacy-import-clear-existing" checked style="width: auto; cursor: pointer;">
          <span>移轉前先清除目前已存在的資料（讀者、書籍、借閱紀錄、身分別），避免歷史借閱等資料重複累增</span>
        </label>
        <button id="btn-legacy-import" class="btn btn-outline" style="color: var(--danger); border-color: var(--danger);" onclick="triggerLegacyImport()">重新讀取 data/ 舊資料庫</button>
        <div id="legacy-import-status" style="display: none; margin-top: 0.75rem; padding: 0.75rem 1rem; border-radius: 0.375rem; font-size: 0.9rem;"></div>
      </div>
    </section>
  </main>

  <!-- 系統全站頁尾資訊 -->
  <footer class="no-print" style="background: #1e293b; color: #94a3b8; padding: 1.25rem 1.5rem; margin-top: auto; border-top: 1px solid #334155; font-size: 0.85rem;">
    <div style="max-width: 1400px; margin: 0 auto; display: flex; justify-content: space-between; align-items: center; flex-wrap: wrap; gap: 0.75rem;">
      <div style="display: flex; align-items: center; gap: 0.6rem; flex-wrap: wrap;">
        <span style="font-weight: 600; color: #f8fafc;">📚 現代化圖書管理借還系統</span>
        <span style="color: #475569;">|</span>
        <span>版本：<code style="background: #0f172a; padding: 0.15rem 0.4rem; border-radius: 0.25rem; color: #38bdf8;">0.12</code></span>
        <span style="color: #475569;">|</span>
        <a id="github-releases-link" href="https://github.com/prolin99/librust/releases" target="_blank" rel="noopener noreferrer" title="前往 GitHub Releases 下載頁面" style="display: inline-flex; align-items: center; gap: 0.35rem; color: #38bdf8; text-decoration: none; font-weight: 500;">
          <svg height="17" width="17" viewBox="0 0 16 16" fill="currentColor" style="display: inline-block; vertical-align: middle;">
            <path d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27.68 0 1.36.09 2 .27 1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.013 8.013 0 0016 8c0-4.42-3.58-8-8-8z"/>
          </svg>
          <span>GitHub Releases</span>
        </a>
        <a id="update-notify-badge" href="https://github.com/prolin99/librust/releases" target="_blank" rel="noopener noreferrer" title="點擊前往下載最新版本" style="display: none; align-items: center; gap: 0.3rem; background: #f59e0b; color: #0f172a; padding: 0.15rem 0.55rem; border-radius: 9999px; font-size: 0.78rem; font-weight: 700; text-decoration: none; box-shadow: 0 0 0 2px rgba(245, 158, 11, 0.35);">
          <span>🔔 發現新版</span>
        </a>
        <button id="btn-apply-update-footer" type="button" onclick="applySystemUpdate()" style="display: none; align-items: center; gap: 0.3rem; background: #10b981; color: #ffffff; border: none; padding: 0.2rem 0.65rem; border-radius: 9999px; font-size: 0.78rem; font-weight: 700; cursor: pointer;">
          <span>⬇️ 一鍵自動更新</span>
        </button>
      </div>
      <div style="display: flex; align-items: center; gap: 0.35rem; flex-wrap: wrap;">
        <span>製作者：</span>
        <a href="mailto:prolin99@gmail.com" title="寄信聯絡作者" style="display: inline-flex; align-items: center; gap: 0.35rem; color: #60a5fa; text-decoration: none; font-weight: 500;">
          <svg height="15" width="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" style="display: inline-block; vertical-align: middle;">
            <rect x="2" y="4" width="20" height="16" rx="2"></rect>
            <path d="m22 7-8.97 5.7a1.94 1.94 0 0 1-2.06 0L2 7"></path>
          </svg>
          <span>prolin99@gmail.com</span>
        </a>
        <span>使用 AI 撰寫 Rust 單機版圖書管理系統</span>
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
        <label style="font-size: 0.85rem; font-weight: 600;">書籍條碼 / 登錄號 <span id="bm-id-hint" style="color: #64748b; font-weight: normal; font-size: 0.8rem;">(留空則由系統於儲存時自動派發)</span></label>
        <input type="text" id="bm-id" placeholder="留空由系統自動派發，或掃描現有條碼">
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
      <div style="display: flex; gap: 0.5rem; margin-bottom: 0.35rem;">
        <div style="flex: 1;">
          <label style="font-size: 0.85rem; font-weight: 600; display: flex; justify-content: space-between; align-items: center;">
            <span>分類號</span>
            <span id="bm-category-hint" style="font-size: 0.78rem; font-weight: 600; color: var(--primary);"></span>
          </label>
          <input type="text" id="bm-category" placeholder="例：300. 或 801" oninput="updateBookModalCategoryHint()">
        </div>
        <div style="flex: 1;">
          <label style="font-size: 0.85rem; font-weight: 600;">存放位置/櫃號</label>
          <input type="text" id="bm-location">
        </div>
      </div>
      <div id="bm-category-legend" style="font-size: 0.76rem; color: var(--text-muted); background: #f8fafc; border: 1px solid var(--border); border-radius: 0.375rem; padding: 0.4rem 0.6rem; margin-bottom: 0.75rem; line-height: 1.5;">
        大分類對照：載入中...
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

  <!-- 書籍大分類管理 Modal -->
  <div id="category-modal" class="modal-overlay">
    <div class="modal" style="max-width: 680px; width: 95%;">
      <div class="modal-header">
        <h3>🏷️ 書籍大分類代碼管理</h3>
        <button class="btn btn-outline btn-sm" onclick="closeModal('category-modal')">✕</button>
      </div>
      <p style="font-size: 0.83rem; color: var(--text-muted); margin-bottom: 0.85rem;">
        系統會依據書籍分類號的<strong>開頭字元（前綴，不分大小寫）</strong>自動歸類，例如分類號 <code>300</code>、<code>300.</code>、<code>310.</code> 皆歸屬代碼 <code>3</code>。可自由修改名稱或新增自訂代碼（如 <code>A</code> 外國）。
      </p>
      <div style="display: flex; gap: 0.5rem; flex-wrap: wrap; align-items: flex-end; background: #f8fafc; border: 1px solid var(--border); border-radius: 0.375rem; padding: 0.75rem; margin-bottom: 0.85rem;">
        <div style="width: 90px;">
          <label style="font-size: 0.8rem; font-weight: 600; display: block;">代碼 *</label>
          <input type="text" id="cat-m-code" placeholder="例: 3 或 A" style="text-transform: uppercase;">
        </div>
        <div style="flex: 1; min-width: 140px;">
          <label style="font-size: 0.8rem; font-weight: 600; display: block;">分類名稱 *</label>
          <input type="text" id="cat-m-name" placeholder="例: 自然科學 / 外國">
        </div>
        <div style="width: 80px;">
          <label style="font-size: 0.8rem; font-weight: 600; display: block;">排序</label>
          <input type="number" id="cat-m-order" value="10">
        </div>
        <div style="flex: 1; min-width: 120px;">
          <label style="font-size: 0.8rem; font-weight: 600; display: block;">備註</label>
          <input type="text" id="cat-m-note" placeholder="選填">
        </div>
        <div style="display: flex; gap: 0.35rem;">
          <button type="button" class="btn btn-primary btn-sm" onclick="saveCategoryFromModal()">儲存</button>
          <button type="button" class="btn btn-outline btn-sm" onclick="resetCategoryForm()">清空</button>
        </div>
      </div>
      <div class="table-container" style="max-height: 320px; overflow-y: auto;">
        <table id="categories-manage-table">
          <thead>
            <tr>
              <th style="width: 75px;">代碼</th>
              <th>分類名稱</th>
              <th style="width: 65px;">排序</th>
              <th>備註</th>
              <th style="width: 125px;">操作</th>
            </tr>
          </thead>
          <tbody></tbody>
        </table>
      </div>
      <div style="display: flex; justify-content: flex-end; margin-top: 1rem;">
        <button class="btn btn-outline" onclick="closeModal('category-modal')">關閉</button>
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
        <label style="font-size: 0.85rem; font-weight: 600;">讀者證號 / 學號 <span id="rm-id-hint" style="color: #64748b; font-weight: normal; font-size: 0.8rem;">(留空則由系統於儲存時自動派發)</span></label>
        <input type="text" id="rm-id" placeholder="留空由系統自動派發，或手動輸入證號">
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
      <div style="margin-bottom: 0.75rem;">
        <label style="font-size: 0.85rem; font-weight: 600;">備註</label>
        <input type="text" id="rm-note" placeholder="選填，例如退證紀錄或特殊說明">
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
      if (tabId === 'settings') { loadSettings(); loadBackupStatus(); }
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

    function exportActiveLoans() {
      const q = (document.getElementById('active-loans-search')?.value || '').trim();
      const filter = document.getElementById('active-loans-status-filter')?.value || '';
      const url = `/api/export/active-loans.csv?status=${encodeURIComponent(filter)}&q=${encodeURIComponent(q)}`;
      const a = document.createElement('a');
      a.href = url;
      a.download = '';
      document.body.appendChild(a);
      a.click();
      document.body.removeChild(a);
    }

    function changeActiveLoansPageSize(val) {
      const parsed = parseInt(val, 10);
      activeLoansPageSize = Math.min(200, Math.max(1, Number.isNaN(parsed) || parsed <= 0 ? 20 : parsed));
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

    // === 3. 書籍管理 (含多筆分頁與大分類管理) ===
    let categoriesCache = [];
    let booksAllList = [];
    let booksCurrentPage = 1;
    let booksPageSize = 20;

    function resolveCategoryMatch(rawCategory) {
      const normalized = (rawCategory || '').trim().toUpperCase();
      if (!normalized || categoriesCache.length === 0) return null;
      let best = null;
      for (const c of categoriesCache) {
        const code = (c.code || '').trim().toUpperCase();
        if (code && normalized.startsWith(code)) {
          if (!best || code.length > (best.code || '').trim().length) {
            best = c;
          }
        }
      }
      return best;
    }

    function formatBookCategoryCell(rawCategory) {
      const trimmed = (rawCategory || '').trim();
      if (!trimmed) return '-';
      const match = resolveCategoryMatch(trimmed);
      if (match) {
        return `<code>${escapeHtml(trimmed)}</code> <span class="badge badge-info">${escapeHtml(match.name)}</span>`;
      }
      return `<code>${escapeHtml(trimmed)}</code>`;
    }

    function updateCategoryLegendAndFilter() {
      const sel = document.getElementById('books-category-filter');
      if (sel) {
        const prev = sel.value;
        const opts = ['<option value="">全部分類</option>'];
        categoriesCache.forEach(c => {
          opts.push(`<option value="${escapeHtml(c.code)}">${escapeHtml(c.code)} - ${escapeHtml(c.name)}</option>`);
        });
        opts.push('<option value="__EMPTY__">未分類（空白）</option>');
        opts.push('<option value="__OTHER__">其他（未定義代碼）</option>');
        sel.innerHTML = opts.join('');
        if ([...sel.options].some(o => o.value === prev)) {
          sel.value = prev;
        }
      }

      const legend = document.getElementById('bm-category-legend');
      if (legend) {
        if (categoriesCache.length === 0) {
          legend.textContent = '大分類對照：尚未設定大分類代碼';
        } else {
          const items = categoriesCache.map(c => `<strong>${escapeHtml(c.code)}</strong> ${escapeHtml(c.name)}`).join(' ｜ ');
          legend.innerHTML = `大分類對照：${items}`;
        }
      }
      updateBookModalCategoryHint();
    }

    function updateBookModalCategoryHint() {
      const input = document.getElementById('bm-category');
      const hint = document.getElementById('bm-category-hint');
      if (!input || !hint) return;
      const val = input.value.trim();
      if (!val) {
        hint.textContent = '';
        return;
      }
      const match = resolveCategoryMatch(val);
      if (match) {
        hint.style.color = 'var(--primary)';
        hint.textContent = `→ ${match.code} ${match.name}`;
      } else {
        hint.style.color = 'var(--text-muted)';
        hint.textContent = '→ 未定義大分類';
      }
    }

    async function loadCategories() {
      try {
        const res = await fetch('/api/categories');
        const data = await res.json();
        if (data.success && data.data) {
          categoriesCache = data.data;
          updateCategoryLegendAndFilter();
          renderCategoriesManageTable();
        }
      } catch (e) {
        console.error('載入大分類失敗:', e);
      }
    }

    function renderCategoriesManageTable() {
      const tbody = document.querySelector('#categories-manage-table tbody');
      if (!tbody) return;
      if (categoriesCache.length === 0) {
        tbody.innerHTML = '<tr><td colspan="5" style="text-align: center; color: var(--text-muted); padding: 1.25rem;">尚無大分類設定</td></tr>';
        return;
      }
      tbody.innerHTML = categoriesCache.map(c => {
        const jsCat = JSON.stringify(c);
        const jsCode = JSON.stringify(c.code);
        return `
          <tr>
            <td><code><strong>${escapeHtml(c.code)}</strong></code></td>
            <td>${escapeHtml(c.name)}</td>
            <td>${c.sort_order}</td>
            <td style="font-size: 0.85rem; color: var(--text-muted);">${escapeHtml(c.note || '-')}</td>
            <td style="white-space: nowrap;">
              <button type="button" class="btn btn-outline btn-sm" onclick='editCategoryForm(${jsCat})'>編輯</button>
              <button type="button" class="btn btn-outline btn-sm" style="color: var(--danger); border-color: var(--danger);" onclick='deleteCategoryItem(${jsCode})'>刪除</button>
            </td>
          </tr>
        `;
      }).join('');
    }

    async function openCategoryManagerModal() {
      await loadCategories();
      resetCategoryForm();
      openModal('category-modal');
    }

    function editCategoryForm(cat) {
      const codeInput = document.getElementById('cat-m-code');
      if (codeInput) {
        codeInput.value = cat.code;
        codeInput.readOnly = true;
      }
      document.getElementById('cat-m-name').value = cat.name || '';
      document.getElementById('cat-m-order').value = cat.sort_order ?? 0;
      document.getElementById('cat-m-note').value = cat.note || '';
    }

    function resetCategoryForm() {
      const codeInput = document.getElementById('cat-m-code');
      if (codeInput) {
        codeInput.value = '';
        codeInput.readOnly = false;
      }
      const nameInput = document.getElementById('cat-m-name');
      if (nameInput) nameInput.value = '';
      const orderInput = document.getElementById('cat-m-order');
      if (orderInput) orderInput.value = categoriesCache.length;
      const noteInput = document.getElementById('cat-m-note');
      if (noteInput) noteInput.value = '';
    }

    async function saveCategoryFromModal() {
      const code = document.getElementById('cat-m-code').value.trim().toUpperCase();
      const name = document.getElementById('cat-m-name').value.trim();
      const sortOrder = parseInt(document.getElementById('cat-m-order').value, 10) || 0;
      const note = document.getElementById('cat-m-note').value.trim();
      if (!code || !name) {
        alert('請填寫分類代碼與分類名稱！');
        return;
      }
      const res = await fetch('/api/categories', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ code, name, sort_order: sortOrder, note }),
      });
      const data = await res.json();
      if (data.success) {
        await loadCategories();
        resetCategoryForm();
        renderBooksTable();
      } else {
        alert(data.message || '儲存大分類失敗');
      }
    }

    async function deleteCategoryItem(code) {
      if (!confirm(`確定要刪除大分類代碼「${code}」嗎？（不會刪除既有書籍資料）`)) return;
      const res = await fetch(`/api/categories/${encodeURIComponent(code)}`, { method: 'DELETE' });
      const data = await res.json();
      if (data.success) {
        await loadCategories();
        renderBooksTable();
      } else {
        alert(data.message || '刪除大分類失敗');
      }
    }

    async function loadBooks() {
      if (categoriesCache.length === 0) {
        await loadCategories();
      }
      const q = document.getElementById('books-search').value.trim();
      const st = document.getElementById('books-status-filter').value;
      const cat = document.getElementById('books-category-filter')?.value || '';
      const res = await fetch(`/api/books?q=${encodeURIComponent(q)}&status=${encodeURIComponent(st)}&category=${encodeURIComponent(cat)}`);
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
      const parsed = parseInt(val, 10);
      booksPageSize = Math.min(200, Math.max(1, Number.isNaN(parsed) || parsed <= 0 ? 20 : parsed));
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

    function updatePageJumpSelect(selectEl, currentPage, totalPages) {
      if (!selectEl) return;
      const tp = Math.max(1, totalPages);
      if (selectEl.dataset.totalPages !== String(tp)) {
        const frag = document.createDocumentFragment();
        for (let p = 1; p <= tp; p++) {
          const opt = document.createElement('option');
          opt.value = String(p);
          opt.textContent = `第 ${p} / ${tp} 頁`;
          frag.appendChild(opt);
        }
        selectEl.innerHTML = '';
        selectEl.appendChild(frag);
        selectEl.dataset.totalPages = String(tp);
      }
      selectEl.value = String(Math.max(1, Math.min(currentPage, tp)));
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
      updatePageJumpSelect(indTop, booksCurrentPage, totalPages);
      const indBtm = document.getElementById('books-page-indicator-bottom');
      updatePageJumpSelect(indBtm, booksCurrentPage, totalPages);

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
          <td>${formatBookCategoryCell(book.category)}</td>
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
      const idInput = document.getElementById('bm-id');
      const idHint = document.getElementById('bm-id-hint');
      if (book) {
        idInput.value = book.id;
        idInput.readOnly = true;
        if (idHint) idHint.textContent = '(不可變更)';
      } else {
        idInput.value = '';
        idInput.readOnly = false;
        if (idHint) idHint.textContent = '(留空則由系統於儲存時自動派發)';
      }
      document.getElementById('bm-title').value = book ? book.title : '';
      document.getElementById('bm-author').value = book ? book.author : '';
      document.getElementById('bm-publisher').value = book ? book.publisher : '';
      document.getElementById('bm-category').value = book ? book.category : '';
      document.getElementById('bm-location').value = book ? book.location : '';
      document.getElementById('bm-isbn').value = book ? book.isbn : '';
      document.getElementById('bm-description').value = book ? book.description : '';
      document.getElementById('bm-cover-url').value = book ? book.cover_url : '';
      document.getElementById('bm-created-at').value = book ? book.created_at : '';
      if (categoriesCache.length === 0) {
        await loadCategories();
      } else {
        updateBookModalCategoryHint();
      }

      // 重設來源標籤與翻譯按鈕
      const srcBar = document.getElementById('bm-isbn-source-bar');
      if (srcBar) srcBar.style.display = 'none';
      const transBtn = document.getElementById('btn-translate-to-zh');
      if (transBtn) transBtn.style.display = 'none';

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
      const isEdit = document.getElementById('bm-id').readOnly;
      const rawId = document.getElementById('bm-id').value.trim();
      const title = document.getElementById('bm-title').value.trim();
      if (!title) { alert('請填寫書名！'); return; }
      if (isEdit && !rawId) { alert('書籍條碼不可為空！'); return; }

      const book = {
        id: rawId,
        title: title,
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

      const url = isEdit ? `/api/books/${encodeURIComponent(rawId)}` : '/api/books';
      const method = isEdit ? 'PUT' : 'POST';

      try {
        const res = await fetch(url, {
          method: method,
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify(book)
        });
        const data = await res.json();
        if (data.success) {
          closeModal('book-modal');
          loadBooks();
        } else {
          alert(data.message || '儲存書籍失敗');
        }
      } catch (e) {
        alert('網路請求失敗: ' + e);
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
      const parsed = parseInt(val, 10);
      readersPageSize = Math.min(200, Math.max(1, Number.isNaN(parsed) || parsed <= 0 ? 20 : parsed));
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
      updatePageJumpSelect(indTop, readersCurrentPage, totalPages);
      const indBtm = document.getElementById('readers-page-indicator-bottom');
      updatePageJumpSelect(indBtm, readersCurrentPage, totalPages);

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
        tbody.innerHTML = '<tr><td colspan="10" style="text-align: center; color: var(--text-muted); padding: 2.5rem;">無符合搜尋條件之讀者資料</td></tr>';
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
          <td style="max-width: 180px; color: var(--text-muted); font-size: 0.85rem;">${escapeHtml(r.note || '-')}</td>
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
      const idInput = document.getElementById('rm-id');
      const idHint = document.getElementById('rm-id-hint');
      if (reader) {
        idInput.value = reader.id;
        idInput.readOnly = true;
        if (idHint) idHint.textContent = '(不可變更)';
      } else {
        idInput.value = '';
        idInput.readOnly = false;
        if (idHint) idHint.textContent = '(留空則由系統於儲存時自動派發)';
      }
      document.getElementById('rm-name').value = reader ? reader.name : '';
      document.getElementById('rm-class').value = reader ? reader.class_name : '';
      document.getElementById('rm-grade').value = reader ? reader.grade : '';
      document.getElementById('rm-phone').value = reader ? reader.phone : '';
      document.getElementById('rm-email').value = reader ? reader.email : '';
      document.getElementById('rm-status').value = reader ? reader.status : 'active';
      document.getElementById('rm-note').value = reader ? (reader.note || '') : '';
      document.getElementById('rm-created-at').value = reader ? reader.created_at : '';
      loadRolesForSelect(reader ? reader.role_id : '01');

      openModal('reader-modal');
    }

    function editReader(reader) { openReaderModal(reader); }

    async function saveReaderFromModal() {
      const isEdit = document.getElementById('rm-id').readOnly;
      const rawId = document.getElementById('rm-id').value.trim();
      const name = document.getElementById('rm-name').value.trim();
      if (!name) { alert('請填寫讀者姓名！'); return; }
      if (isEdit && !rawId) { alert('讀者證號不可為空！'); return; }

      const reader = {
        id: rawId,
        name: name,
        role_id: document.getElementById('rm-role-id').value,
        role_name: null,
        max_books: null,
        borrow_days: null,
        class_name: document.getElementById('rm-class').value.trim(),
        grade: document.getElementById('rm-grade').value.trim(),
        phone: document.getElementById('rm-phone').value.trim(),
        email: document.getElementById('rm-email').value.trim(),
        status: document.getElementById('rm-status').value,
        note: document.getElementById('rm-note').value.trim(),
        created_at: document.getElementById('rm-created-at').value,
        active_loans_count: 0,
      };

      const url = isEdit ? `/api/readers/${encodeURIComponent(rawId)}` : '/api/readers';
      const method = isEdit ? 'PUT' : 'POST';

      try {
        const res = await fetch(url, {
          method: method,
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify(reader)
        });
        const data = await res.json();
        if (data.success) {
          closeModal('reader-modal');
          loadReaders();
        } else {
          alert(data.message || '儲存讀者失敗');
        }
      } catch (e) {
        alert('網路請求失敗: ' + e);
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

    let currentPrintType = 'barcodes'; // 'barcodes' 或 'cards'
    let cachedBarcodeWidth = 48;
    let cachedBarcodeHeight = 40;
    let cachedCardWidth = 86;
    let cachedCardHeight = 54;
    let resizeTimer = null;

    async function previewBatchCards(switchInputs = true) {
      currentPrintType = 'cards';
      const dimTitle = document.getElementById('print-dim-title');
      if (dimTitle) dimTitle.textContent = '📏 借書證尺寸設定：';

      if (switchInputs) {
        document.getElementById('set-barcode-w').value = cachedCardWidth;
        document.getElementById('set-barcode-h').value = cachedCardHeight;
      }
      const wMm = parseFloat(document.getElementById('set-barcode-w')?.value) || cachedCardWidth;
      const hMm = parseFloat(document.getElementById('set-barcode-h')?.value) || cachedCardHeight;
      cachedCardWidth = wMm;
      cachedCardHeight = hMm;

      const res = await fetch('/api/readers');
      const data = await res.json();
      const settingsRes = await fetch('/api/settings');
      const sData = await settingsRes.json();
      const s = sData.data || {};

      const readers = filterPrintItems(data.data || [], 'id', ['name', 'class_name', 'role_name']);
      const countLabel = document.getElementById('print-count-label');
      if (countLabel) countLabel.textContent = `📋 預覽借書證：共 ${readers.length} 筆符合 (${wMm}×${hMm} mm)`;

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
          <div class="print-tag-item" style="width: ${wMm}mm; height: ${hMm}mm; min-width: ${wMm}mm; min-height: ${hMm}mm; max-width: ${wMm}mm; max-height: ${hMm}mm; border: 1px solid #000; border-radius: 4px; padding: 3mm; background: white; page-break-inside: avoid; display: flex; flex-direction: column; justify-content: space-between; box-sizing: border-box; overflow: hidden;">
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

    async function previewBatchBarcodes(switchInputs = true) {
      currentPrintType = 'barcodes';
      const dimTitle = document.getElementById('print-dim-title');
      if (dimTitle) dimTitle.textContent = '📏 書籍標籤尺寸設定：';

      const settingsRes = await fetch('/api/settings');
      const sData = await settingsRes.json();
      const s = sData.data || {};

      if (s.barcode_width_mm && switchInputs && cachedBarcodeWidth === 48) {
        cachedBarcodeWidth = s.barcode_width_mm;
      }
      if (s.barcode_height_mm && switchInputs && cachedBarcodeHeight === 40) {
        cachedBarcodeHeight = s.barcode_height_mm;
      }

      if (switchInputs) {
        document.getElementById('set-barcode-w').value = cachedBarcodeWidth;
        document.getElementById('set-barcode-h').value = cachedBarcodeHeight;
      }
      const wMm = parseFloat(document.getElementById('set-barcode-w')?.value) || cachedBarcodeWidth;
      const hMm = parseFloat(document.getElementById('set-barcode-h')?.value) || cachedBarcodeHeight;
      cachedBarcodeWidth = wMm;
      cachedBarcodeHeight = hMm;

      const res = await fetch('/api/books');
      const data = await res.json();

      const books = filterPrintItems(data.data || [], 'id', ['title', 'author', 'category', 'location']);
      const countLabel = document.getElementById('print-count-label');
      if (countLabel) countLabel.textContent = `🏷️ 預覽書籍標籤：共 ${books.length} 筆符合 (${wMm}×${hMm} mm)`;

      const area = document.getElementById('print-area');
      area.innerHTML = '';
      if (books.length === 0) {
        area.innerHTML = '<p style="color: var(--text-muted); text-align: center; margin-top: 5rem;">查無符合所設範圍之書籍資料！請調整起訖編號或關鍵字。</p>';
        return;
      }

      // 背景自動保存最新貼紙尺寸至 settings
      if (s && (!isNaN(wMm) || !isNaN(hMm))) {
        s.barcode_width_mm = wMm;
        s.barcode_height_mm = hMm;
        fetch('/api/settings', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify(s)
        }).catch(err => console.error(err));
      }

      // 依高度動態調整條碼圖形高度
      const bcHeight = Math.max(14, Math.min(36, Math.round(hMm * 0.42)));

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

    // 即時動態修改尺寸（0毫秒延遲更新畫面上元素，並防抖重新排版）
    function onBarcodeDimensionChange() {
      const w = parseFloat(document.getElementById('set-barcode-w')?.value);
      const h = parseFloat(document.getElementById('set-barcode-h')?.value);
      if (isNaN(w) || isNaN(h) || w <= 0 || h <= 0) return;

      if (currentPrintType === 'cards') {
        cachedCardWidth = w;
        cachedCardHeight = h;
      } else {
        cachedBarcodeWidth = w;
        cachedBarcodeHeight = h;
      }

      // 1. 立即更新畫面上既有 .print-tag-item 樣式
      const items = document.querySelectorAll('.print-tag-item');
      items.forEach(el => {
        el.style.width = w + 'mm';
        el.style.minWidth = w + 'mm';
        el.style.maxWidth = w + 'mm';
        el.style.height = h + 'mm';
        el.style.minHeight = h + 'mm';
        el.style.maxHeight = h + 'mm';
      });

      // 2. 顯示綠色即時更新標籤
      const reminder = document.getElementById('barcode-dim-reminder');
      if (reminder) {
        reminder.style.display = 'inline-flex';
        reminder.style.background = '#f0fdf4';
        reminder.style.border = '1px solid #86efac';
        reminder.style.color = '#15803d';
        reminder.innerHTML = `✔ 已即時更新尺寸：<strong>${w} × ${h} mm</strong>`;
      }

      // 3. 防抖 300ms 重新精確排版
      clearTimeout(resizeTimer);
      resizeTimer = setTimeout(() => {
        triggerInstantPreview();
      }, 300);
    }

    function triggerInstantPreview() {
      if (currentPrintType === 'cards') {
        previewBatchCards(false);
      } else {
        previewBatchBarcodes(false);
      }
    }

    function onSpineCheckboxChange() {
      // 1. 勾選/取消立即重新排版渲染預覽
      triggerInstantPreview();

      // 2. 背景同步更新至設定中的 show_spine_tag
      const optSpine = document.getElementById('print-opt-spine');
      if (optSpine) {
        fetch('/api/settings').then(r => r.json()).then(sData => {
          if (sData.success && sData.data) {
            const s = sData.data;
            s.show_spine_tag = optSpine.checked;
            fetch('/api/settings', {
              method: 'POST',
              headers: { 'Content-Type': 'application/json' },
              body: JSON.stringify(s)
            }).catch(e => console.error(e));
          }
        }).catch(e => console.error(e));
      }
    }

    let printFilterTimer = null;
    function onPrintOptionDebounced() {
      clearTimeout(printFilterTimer);
      printFilterTimer = setTimeout(() => {
        triggerInstantPreview();
      }, 350);
    }

    function doDirectPrint() {
      const w = parseFloat(document.getElementById('set-barcode-w')?.value);
      const h = parseFloat(document.getElementById('set-barcode-h')?.value);
      if (!isNaN(w) && !isNaN(h) && w > 0 && h > 0) {
        document.querySelectorAll('.print-tag-item').forEach(el => {
          el.style.width = w + 'mm';
          el.style.minWidth = w + 'mm';
          el.style.maxWidth = w + 'mm';
          el.style.height = h + 'mm';
          el.style.minHeight = h + 'mm';
          el.style.maxHeight = h + 'mm';
        });
      }
      window.print();
    }

    // === 8. 統計儀表板 ===
    let statsRangeInitialized = false;

    function formatDateYmd(date) {
      const y = date.getFullYear();
      const m = String(date.getMonth() + 1).padStart(2, '0');
      const d = String(date.getDate()).padStart(2, '0');
      return `${y}-${m}-${d}`;
    }

    function setStatsPreset(preset, autoLoad = true) {
      const now = new Date();
      const startEl = document.getElementById('stats-start-date');
      const endEl = document.getElementById('stats-end-date');
      if (!startEl || !endEl) return;

      if (preset === 'all') {
        startEl.value = '';
        endEl.value = '';
      } else if (preset === '6m') {
        const start = new Date(now.getFullYear(), now.getMonth() - 5, 1);
        startEl.value = formatDateYmd(start);
        endEl.value = formatDateYmd(now);
      } else if (preset === '1y') {
        const start = new Date(now.getFullYear(), now.getMonth() - 11, 1);
        startEl.value = formatDateYmd(start);
        endEl.value = formatDateYmd(now);
      } else if (preset === 'year') {
        const start = new Date(now.getFullYear(), 0, 1);
        startEl.value = formatDateYmd(start);
        endEl.value = formatDateYmd(now);
      }
      if (autoLoad) loadRangeStats();
    }

    function getRangeStatsParams() {
      const startDate = (document.getElementById('stats-start-date')?.value || '').trim();
      const endDate = (document.getElementById('stats-end-date')?.value || '').trim();
      const rawLimit = parseInt(document.getElementById('stats-limit')?.value ?? '20', 10);
      const limit = Number.isNaN(rawLimit) || rawLimit < 0 ? 20 : rawLimit;
      const params = new URLSearchParams();
      if (startDate) params.set('start_date', startDate);
      if (endDate) params.set('end_date', endDate);
      params.set('limit', String(limit));
      return { startDate, endDate, limit, queryString: params.toString() };
    }

    function exportRangeStatsCsv(kind) {
      const { queryString } = getRangeStatsParams();
      const url = `/api/export/stats/${kind}.csv?${queryString}`;
      window.location.href = url;
    }

    function renderMonthlyChart(monthlyStats) {
      const container = document.getElementById('monthly-borrow-chart');
      if (!container) return;

      if (!monthlyStats || monthlyStats.length === 0) {
        container.innerHTML = '<p style="text-align: center; color: var(--text-muted); padding: 3rem 0;">此時間範圍內尚無借閱紀錄 📭</p>';
        return;
      }

      const maxVal = Math.max(
        1,
        ...monthlyStats.map(m => Math.max(m.borrow_visits || 0, m.borrow_books || 0))
      );
      // 取漂亮的 Y 軸上限
      const rawStep = maxVal / 4;
      const mag = Math.pow(10, Math.floor(Math.log10(Math.max(1, rawStep))));
      const niceStep = Math.ceil(rawStep / mag) * mag;
      const yMax = niceStep * 4;

      const padLeft = 48;
      const padRight = 20;
      const padTop = 26;
      const padBottom = 52;
      const chartHeight = 200;
      const groupWidth = Math.max(58, Math.min(90, Math.floor(960 / Math.max(1, monthlyStats.length))));
      const svgWidth = Math.max(600, padLeft + padRight + monthlyStats.length * groupWidth);
      const svgHeight = padTop + chartHeight + padBottom;
      const barWidth = Math.min(22, Math.floor((groupWidth - 18) / 2));
      const barGap = 4;

      let gridHtml = '';
      for (let i = 0; i <= 4; i++) {
        const val = Math.round((yMax / 4) * i);
        const y = padTop + chartHeight - (chartHeight * i) / 4;
        gridHtml += `
          <line x1="${padLeft}" y1="${y}" x2="${svgWidth - padRight}" y2="${y}" stroke="#e2e8f0" stroke-dasharray="${i === 0 ? '' : '3,3'}" />
          <text x="${padLeft - 8}" y="${y + 4}" text-anchor="end" font-size="11" fill="#64748b">${val}</text>
        `;
      }

      let barsHtml = '';
      monthlyStats.forEach((m, idx) => {
        const cx = padLeft + idx * groupWidth + groupWidth / 2;
        const visits = m.borrow_visits || 0;
        const books = m.borrow_books || 0;
        const hVisits = (visits / yMax) * chartHeight;
        const hBooks = (books / yMax) * chartHeight;
        const xVisits = cx - barWidth - barGap / 2;
        const xBooks = cx + barGap / 2;
        const yVisits = padTop + chartHeight - hVisits;
        const yBooks = padTop + chartHeight - hBooks;

        barsHtml += `
          <g>
            <rect x="${xVisits}" y="${yVisits}" width="${barWidth}" height="${Math.max(0, hVisits)}" rx="3" fill="#10b981">
              <title>${escapeHtml(m.month)} 借閱人次：${visits} 人次</title>
            </rect>
            ${visits > 0 ? `<text x="${xVisits + barWidth / 2}" y="${yVisits - 5}" text-anchor="middle" font-size="10" font-weight="600" fill="#059669">${visits}</text>` : ''}
            <rect x="${xBooks}" y="${yBooks}" width="${barWidth}" height="${Math.max(0, hBooks)}" rx="3" fill="#2563eb">
              <title>${escapeHtml(m.month)} 借閱冊數：${books} 冊</title>
            </rect>
            ${books > 0 ? `<text x="${xBooks + barWidth / 2}" y="${yBooks - 5}" text-anchor="middle" font-size="10" font-weight="600" fill="#1d4ed8">${books}</text>` : ''}
            <text x="${cx}" y="${padTop + chartHeight + 18}" text-anchor="middle" font-size="11" fill="#334155" font-weight="500">${escapeHtml(m.month)}</text>
          </g>
        `;
      });

      container.innerHTML = `
        <svg width="${svgWidth}" height="${svgHeight}" viewBox="0 0 ${svgWidth} ${svgHeight}" style="display: block; margin: 0 auto;">
          ${gridHtml}
          ${barsHtml}
        </svg>
      `;
    }

    async function loadRangeStats() {
      const { limit, queryString } = getRangeStatsParams();
      const rankLabel = limit === 0 ? '全部' : `前 ${limit} 筆`;
      const rb = document.getElementById('reader-rank-badge');
      const bb = document.getElementById('book-rank-badge');
      if (rb) rb.textContent = rankLabel;
      if (bb) bb.textContent = rankLabel;

      try {
        const res = await fetch(`/api/stats/range?${queryString}`);
        const data = await res.json();
        if (!data.success || !data.data) return;

        const r = data.data;
        document.getElementById('range-total-visits').textContent = r.total_visits;
        document.getElementById('range-total-books').textContent = r.total_books;
        document.getElementById('range-month-count').textContent = (r.monthly_stats || []).length;

        renderMonthlyChart(r.monthly_stats || []);

        // 月份明細表
        const monthTbody = document.querySelector('#monthly-stats-table tbody');
        if (monthTbody) {
          const list = r.monthly_stats || [];
          if (list.length === 0) {
            monthTbody.innerHTML = '<tr><td colspan="3" style="text-align: center; color: var(--text-muted); padding: 1rem;">此期間無資料</td></tr>';
          } else {
            monthTbody.innerHTML = list.map(m => `
              <tr>
                <td><strong>${escapeHtml(m.month)}</strong></td>
                <td style="color: #059669; font-weight: 600;">${m.borrow_visits} 人次</td>
                <td style="color: #2563eb; font-weight: 600;">${m.borrow_books} 冊</td>
              </tr>
            `).join('') + `
              <tr style="background: #f8fafc; font-weight: bold;">
                <td>合計</td>
                <td style="color: #059669;">${r.total_visits} 人次</td>
                <td style="color: #2563eb;">${r.total_books} 冊</td>
              </tr>
            `;
          }
        }

        // 個人借閱冊數排行
        const readerTbody = document.querySelector('#reader-rank-table tbody');
        if (readerTbody) {
          const list = r.reader_rankings || [];
          if (list.length === 0) {
            readerTbody.innerHTML = '<tr><td colspan="5" style="text-align: center; color: var(--text-muted); padding: 1.5rem;">此時間範圍內尚無個人借閱紀錄</td></tr>';
          } else {
            readerTbody.innerHTML = list.map((item, idx) => `
              <tr>
                <td><strong>#${idx + 1}</strong></td>
                <td><strong>${escapeHtml(item.reader_name)}</strong> <span style="font-size: 0.78rem; color: var(--text-muted);">(${escapeHtml(item.reader_id)})</span></td>
                <td style="font-size: 0.82rem; color: var(--text-muted);">${escapeHtml(item.role_name || '-')}${item.class_name ? ' / ' + escapeHtml(item.class_name) : ''}</td>
                <td style="text-align: right; color: #059669; font-weight: 600;">${item.borrow_visits}</td>
                <td style="text-align: right;"><span class="badge badge-info">${item.borrow_books} 冊</span></td>
              </tr>
            `).join('');
          }
        }

        // 書籍被借次數排行
        const bookTbody = document.querySelector('#book-rank-table tbody');
        if (bookTbody) {
          const list = r.book_rankings || [];
          if (list.length === 0) {
            bookTbody.innerHTML = '<tr><td colspan="4" style="text-align: center; color: var(--text-muted); padding: 1.5rem;">此時間範圍內尚無書籍借閱紀錄</td></tr>';
          } else {
            bookTbody.innerHTML = list.map((item, idx) => `
              <tr>
                <td><strong>#${idx + 1}</strong></td>
                <td><strong>${escapeHtml(item.title)}</strong> <span style="font-size: 0.78rem; color: var(--text-muted);">(${escapeHtml(item.book_id)})</span></td>
                <td style="font-size: 0.82rem; color: var(--text-muted);">${escapeHtml(item.author || '-')}${item.category ? ' / ' + escapeHtml(item.category) : ''}</td>
                <td style="text-align: right;"><span class="badge badge-info">${item.borrow_count} 次</span></td>
              </tr>
            `).join('');
          }
        }
      } catch (e) {
        console.error('載入期間統計失敗:', e);
      }
    }

    async function loadStats() {
      if (!statsRangeInitialized) {
        statsRangeInitialized = true;
        setStatsPreset('1y', false);
      }
      loadRangeStats();

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

    // === 9. 系統設定與密碼變更 ===
    async function loadSettings() {
      const res = await fetch('/api/settings');
      const data = await res.json();
      if (data.success && data.data) {
        const s = data.data;
        document.getElementById('set-lib-name').value = s.library_name;
        document.getElementById('header-library-name').textContent = s.library_name;
        const loginTitle = document.getElementById('login-app-title');
        if (loginTitle) loginTitle.textContent = s.library_name;
        document.getElementById('set-lib-phone').value = s.library_phone;
        document.getElementById('set-lib-address').value = s.library_address;
        if (document.getElementById('set-lib-rules')) {
          document.getElementById('set-lib-rules').value = s.borrow_rules || '';
        }
        if (s.barcode_width_mm) cachedBarcodeWidth = s.barcode_width_mm;
        if (s.barcode_height_mm) cachedBarcodeHeight = s.barcode_height_mm;
        if (document.getElementById('set-barcode-w') && currentPrintType === 'barcodes') {
          document.getElementById('set-barcode-w').value = cachedBarcodeWidth;
        }
        if (document.getElementById('set-barcode-h') && currentPrintType === 'barcodes') {
          document.getElementById('set-barcode-h').value = cachedBarcodeHeight;
        }
        if (document.getElementById('set-show-spine')) {
          document.getElementById('set-show-spine').checked = s.show_spine_tag;
        }
      }
    }

    async function saveSettings() {
      const settings = {
        library_name: document.getElementById('set-lib-name').value.trim(),
        library_phone: document.getElementById('set-lib-phone').value.trim(),
        library_address: document.getElementById('set-lib-address').value.trim(),
        borrow_rules: document.getElementById('set-lib-rules') ? document.getElementById('set-lib-rules').value.trim() : '',
        barcode_width_mm: parseFloat(document.getElementById('set-barcode-w')?.value) || 48,
        barcode_height_mm: parseFloat(document.getElementById('set-barcode-h')?.value) || 40,
        show_spine_tag: document.getElementById('opt-show-spine') ? document.getElementById('opt-show-spine').checked : true,
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
        const loginTitle = document.getElementById('login-app-title');
        if (loginTitle) loginTitle.textContent = settings.library_name;
      }
    }

    async function handleChangePassword() {
      const current_password = document.getElementById('change-pwd-current').value;
      const new_password = document.getElementById('change-pwd-new').value;
      const confirm_password = document.getElementById('change-pwd-confirm').value;

      if (!current_password) {
        showPwdMsg('請輸入目前密碼！', true);
        return;
      }
      if (!new_password) {
        showPwdMsg('請輸入新密碼！', true);
        return;
      }
      if (new_password !== confirm_password) {
        showPwdMsg('新密碼與確認密碼不一致！', true);
        return;
      }
      if (new_password.length < 3) {
        showPwdMsg('新密碼長度建議至少3個字元！', true);
        return;
      }

      try {
        const res = await fetch('/api/auth/change-password', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ current_password, new_password })
        });
        const data = await res.json();
        if (data.success) {
          showPwdMsg(data.message || '密碼已成功更新！', false);
          document.getElementById('change-pwd-current').value = '';
          document.getElementById('change-pwd-new').value = '';
          document.getElementById('change-pwd-confirm').value = '';
        } else {
          showPwdMsg(data.message || '更新失敗！', true);
        }
      } catch (err) {
        showPwdMsg('連線伺服器失敗: ' + err.message, true);
      }
    }

    function showPwdMsg(msg, isError) {
      const msgEl = document.getElementById('change-pwd-msg');
      msgEl.style.display = 'block';
      msgEl.style.background = isError ? '#fee2e2' : '#dcfce7';
      msgEl.style.color = isError ? '#b91c1c' : '#15803d';
      msgEl.style.border = isError ? '1px solid #f87171' : '1px solid #86efac';
      msgEl.textContent = msg;
    }

    async function triggerLegacyImport() {
      const clearCheckbox = document.getElementById('legacy-import-clear-existing');
      const clearExisting = clearCheckbox ? clearCheckbox.checked : false;
      const modeHint = clearExisting
        ? '【先清除目前所有讀者、書籍、借閱紀錄與身分別，再重新匯入】'
        : '【保留現有資料並覆蓋／附加匯入（注意：歷史借閱紀錄可能會重複累增）】';
      if (!confirm(`確定要重新讀取 data/ 目錄下的 DBF 舊資料庫嗎？\n\n本次模式：${modeHint}\n\n⏳ 讀取與移轉可能需要數十秒至數分鐘，請耐心等待，期間切勿關閉視窗。`)) return;

      const btn = document.getElementById('btn-legacy-import');
      const statusEl = document.getElementById('legacy-import-status');
      if (btn) {
        btn.disabled = true;
        btn.textContent = '⏳ 正在讀取舊版 DBF 並移轉資料中，請耐心等待...';
      }
      if (statusEl) {
        statusEl.style.display = 'block';
        statusEl.style.background = '#fef3c7';
        statusEl.style.color = '#92400e';
        statusEl.style.border = '1px solid #f59e0b';
        statusEl.textContent = '⏳ 正在讀取 data/ 舊版 DBF 檔案並寫入資料庫，資料量較大時可能需要數十秒至數分鐘，請耐心等待，切勿關閉視窗或重新整理頁面...';
      }

      try {
        const res = await fetch('/api/system/import-legacy', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ clear_existing: clearExisting }),
        });
        const data = await res.json();
        if (statusEl) {
          if (data.success) {
            statusEl.style.background = '#dcfce7';
            statusEl.style.color = '#15803d';
            statusEl.style.border = '1px solid #86efac';
            statusEl.textContent = `✅ ${data.message}`;
          } else {
            statusEl.style.background = '#fee2e2';
            statusEl.style.color = '#b91c1c';
            statusEl.style.border = '1px solid #f87171';
            statusEl.textContent = `❌ ${data.message}`;
          }
        }
        alert(data.message);
        loadStats();
      } catch (e) {
        const errMsg = '移轉過程發生連線錯誤：' + e.message;
        if (statusEl) {
          statusEl.style.background = '#fee2e2';
          statusEl.style.color = '#b91c1c';
          statusEl.style.border = '1px solid #f87171';
          statusEl.textContent = `❌ ${errMsg}`;
        }
        alert(errMsg);
      } finally {
        if (btn) {
          btn.disabled = false;
          btn.textContent = '重新讀取 data/ 舊資料庫';
        }
      }
    }

    // === 登入與身分認證控制 ===
    async function checkAuthStatus() {
      try {
        const res = await fetch('/api/auth/status');
        const data = await res.json();
        if (data.success && data.data && data.data.logged_in) {
          setLoggedInState(data.data.username || 'admin');
          return true;
        }
      } catch (e) {
        console.error('Check auth status error:', e);
      }
      setLoggedOutState();
      return false;
    }

    function setLoggedInState(username) {
      document.getElementById('login-overlay').style.display = 'none';
      document.getElementById('current-user-display').textContent = '👤 ' + username;
      document.getElementById('user-info-bar').style.display = 'flex';
      loadSettings();
      loadRoles();
      document.getElementById('borrow-reader-id').focus();
    }

    function setLoggedOutState() {
      document.getElementById('login-overlay').style.display = 'flex';
      document.getElementById('user-info-bar').style.display = 'none';
      document.getElementById('login-password').value = '';
      const errEl = document.getElementById('login-error-msg');
      if (errEl) errEl.style.display = 'none';
      setTimeout(() => {
        const pwdInput = document.getElementById('login-password');
        if (pwdInput) pwdInput.focus();
      }, 100);
    }

    async function doLogin() {
      const username = document.getElementById('login-username').value.trim();
      const password = document.getElementById('login-password').value;
      const errEl = document.getElementById('login-error-msg');
      errEl.style.display = 'none';

      if (!username || !password) {
        errEl.textContent = '請輸入帳號與密碼！';
        errEl.style.display = 'block';
        return;
      }

      try {
        const res = await fetch('/api/auth/login', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ username, password })
        });
        const data = await res.json();
        if (data.success && data.data) {
          setLoggedInState(data.data.username);
        } else {
          errEl.textContent = data.message || '帳號或密碼錯誤！';
          errEl.style.display = 'block';
        }
      } catch (err) {
        errEl.textContent = '連線伺服器失敗: ' + err.message;
        errEl.style.display = 'block';
      }
    }

    async function handleLogout() {
      if (!confirm('確定要登出系統嗎？')) return;
      try {
        await fetch('/api/auth/logout', { method: 'POST' });
      } catch (e) {}
      setLoggedOutState();
    }

    async function handleShutdown() {
      if (!confirm('確定要安全關閉圖書管理系統嗎？\n\n關閉後將停止後台服務，本機與區網連線將會中斷。')) return;
      try {
        await fetch('/api/system/shutdown', { method: 'POST' });
      } catch (e) {}
      document.body.innerHTML = `
        <div style="min-height: 100vh; display: flex; align-items: center; justify-content: center; background: #0f172a; color: #f8fafc; font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; padding: 2rem;">
          <div style="text-align: center; max-width: 480px; background: #1e293b; padding: 2.5rem 2rem; border-radius: 1rem; box-shadow: 0 25px 50px -12px rgba(0,0,0,0.5); border: 1px solid #334155;">
            <div style="font-size: 4rem; margin-bottom: 1rem;">📚💤</div>
            <h2 style="font-size: 1.5rem; margin-bottom: 0.75rem; color: #38bdf8;">系統已安全關閉</h2>
            <p style="color: #94a3b8; font-size: 0.95rem; line-height: 1.6; margin-bottom: 1.5rem;">
              資料庫已完整保存，背景伺服器已停止運行。<br>您現在可以安全關閉此瀏覽器分頁。
            </p>
            <div style="font-size: 0.85rem; color: #64748b; border-top: 1px solid #334155; padding-top: 1rem;">
              若需重新使用，請再次執行「圖書管理系統.exe」即可啟動。
            </div>
          </div>
        </div>
      `;
    }

    let latestUpdateInfo = null;

    // 檢查 GitHub 是否有新版本發布
    async function checkForUpdates(manual = false) {
      const infoEl = document.getElementById('settings-update-info');
      if (manual && infoEl) {
        infoEl.textContent = '⏳ 正在向 GitHub Releases 查詢最新版本...';
      }
      try {
        const url = manual ? '/api/system/check-update?force=1' : '/api/system/check-update';
        const res = await fetch(url);
        const data = await res.json();
        if (data.success && data.data) {
          const info = data.data;
          latestUpdateInfo = info;
          const badge = document.getElementById('update-notify-badge');
          const footerBtn = document.getElementById('btn-apply-update-footer');
          const settingsBtn = document.getElementById('btn-apply-update-settings');

          if (info.has_update) {
            const latest = info.latest_version || '新版';
            if (badge) {
              badge.href = info.release_url || 'https://github.com/prolin99/librust/releases';
              badge.innerHTML = `<span>🔔 發現新版 ${escapeHtml(latest)}</span>`;
              badge.style.display = 'inline-flex';
            }
            if (info.download_url) {
              const sizeMb = info.asset_size ? ` (${(info.asset_size / (1024 * 1024)).toFixed(1)} MB)` : '';
              if (footerBtn) {
                footerBtn.innerHTML = `<span>⬇️ 一鍵更新至 ${escapeHtml(latest)}</span>`;
                footerBtn.style.display = 'inline-flex';
              }
              if (settingsBtn) {
                settingsBtn.textContent = `⬇️ 一鍵下載並更新至 ${latest}${sizeMb}`;
                settingsBtn.style.display = 'inline-flex';
              }
              if (infoEl) {
                infoEl.innerHTML = `目前版本：<code>v${escapeHtml(info.current_version)}</code> ｜ 發現新版：<strong>${escapeHtml(latest)}</strong>（附件 <code>${escapeHtml(info.asset_name || 'LibRust.exe')}</code>${sizeMb}），可直接一鍵下載更新並重啟。`;
              }
            } else if (infoEl) {
              infoEl.innerHTML = `目前版本：<code>v${escapeHtml(info.current_version)}</code> ｜ 發現新版：<strong>${escapeHtml(latest)}</strong>（此版本尚未附上 .exe 檔案，請至 <a href="${escapeHtml(info.release_url)}" target="_blank" rel="noopener noreferrer">GitHub Releases</a> 手動下載）。`;
            }
          } else {
            if (footerBtn) footerBtn.style.display = 'none';
            if (settingsBtn) settingsBtn.style.display = 'none';
            if (infoEl) {
              const latestText = info.latest_version ? `（線上最新：${escapeHtml(info.latest_version)}）` : '（目前無法連線至 GitHub 或尚無線上版本）';
              infoEl.innerHTML = `目前版本：<code>v${escapeHtml(info.current_version)}</code>，已是最新版本 ${latestText}`;
            }
          }
        }
      } catch (e) {
        if (manual && infoEl) {
          infoEl.textContent = '無法連線至更新伺服器，請確認網路連線。';
        }
      }
    }

    async function applySystemUpdate() {
      const targetVer = latestUpdateInfo?.latest_version || '最新版本';
      const assetName = latestUpdateInfo?.asset_name || 'LibRust.exe';
      if (!confirm(`確定要從 GitHub 下載並自動更新至 ${targetVer}（${assetName}）嗎？\n\n⏳ 下載與套用更新期間請耐心等待，完成後系統將自動重新啟動並重新整理頁面。`)) {
        return;
      }

      const footerBtn = document.getElementById('btn-apply-update-footer');
      const settingsBtn = document.getElementById('btn-apply-update-settings');
      const statusEl = document.getElementById('apply-update-status');
      if (footerBtn) {
        footerBtn.disabled = true;
        footerBtn.innerHTML = '<span>⏳ 更新下載中...</span>';
      }
      if (settingsBtn) {
        settingsBtn.disabled = true;
        settingsBtn.textContent = '⏳ 正在下載新版執行檔並套用更新，請耐心等待...';
      }
      if (statusEl) {
        statusEl.style.display = 'block';
        statusEl.style.background = '#fef3c7';
        statusEl.style.color = '#92400e';
        statusEl.style.border = '1px solid #f59e0b';
        statusEl.textContent = '⏳ 正在從 GitHub 下載新版執行檔並自動替換，請耐心等待，切勿關閉視窗...';
      }

      try {
        const res = await fetch('/api/system/apply-update', { method: 'POST' });
        const data = await res.json();
        if (!data.success) {
          if (statusEl) {
            statusEl.style.background = '#fee2e2';
            statusEl.style.color = '#b91c1c';
            statusEl.style.border = '1px solid #f87171';
            statusEl.textContent = `❌ ${data.message}`;
          }
          alert(data.message);
          if (footerBtn) {
            footerBtn.disabled = false;
            footerBtn.innerHTML = `<span>⬇️ 一鍵更新至 ${escapeHtml(targetVer)}</span>`;
          }
          if (settingsBtn) {
            settingsBtn.disabled = false;
            settingsBtn.textContent = `⬇️ 一鍵下載並更新至 ${targetVer}`;
          }
          return;
        }

        // 顯示等待重啟畫面並輪詢 /api/system/ping 直到新版啟動完成
        document.body.innerHTML = `
          <div style="display:flex;align-items:center;justify-content:center;min-height:100vh;background:#0f172a;color:#f8fafc;font-family:sans-serif;text-align:center;padding:2rem;">
            <div style="max-width:480px;background:#1e293b;padding:2.5rem;border-radius:1rem;border:1px solid #334155;">
              <div style="font-size:3rem;margin-bottom:1rem;">🚀</div>
              <h2 style="margin-bottom:0.75rem;color:#38bdf8;">系統更新完成，正在重新啟動...</h2>
              <p style="color:#cbd5e1;line-height:1.6;margin-bottom:1.25rem;">${escapeHtml(data.message)}</p>
              <p id="update-reload-hint" style="color:#94a3b8;font-size:0.9rem;">⏳ 正在等待新版伺服器就緒，就緒後將自動重新載入頁面...</p>
            </div>
          </div>
        `;

        let attempts = 0;
        const pollTimer = setInterval(async () => {
          attempts++;
          try {
            const pingRes = await fetch('/api/system/ping', { cache: 'no-store' });
            if (pingRes.ok) {
              const pingData = await pingRes.json();
              if (pingData && pingData.app === 'librust') {
                clearInterval(pollTimer);
                window.location.reload();
                return;
              }
            }
          } catch (_) {
            // 舊程序剛結束、新程序尚未完成綁定，繼續等待
          }
          if (attempts >= 40) {
            clearInterval(pollTimer);
            const hint = document.getElementById('update-reload-hint');
            if (hint) {
              hint.textContent = '重啟等待逾時，請手動重新整理頁面或重新開啟「圖書管理系統.exe」。';
            }
          }
        }, 1500);
      } catch (e) {
        const errMsg = '下載更新時發生連線錯誤：' + e.message;
        if (statusEl) {
          statusEl.style.background = '#fee2e2';
          statusEl.style.color = '#b91c1c';
          statusEl.style.border = '1px solid #f87171';
          statusEl.textContent = `❌ ${errMsg}`;
        }
        alert(errMsg);
        if (footerBtn) footerBtn.disabled = false;
        if (settingsBtn) settingsBtn.disabled = false;
      }
    }

    // === 資料庫備份、還原與定期提醒 ===
    let backupReminderDismissed = false;

    function formatFileSize(bytes) {
      if (!bytes || bytes <= 0) return '0 B';
      if (bytes < 1024) return `${bytes} B`;
      if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
      return `${(bytes / (1024 * 1024)).toFixed(2)} MB`;
    }

    function showBackupStatusMsg(msg, isError = false, isPending = false) {
      const el = document.getElementById('backup-action-status');
      if (!el) return;
      el.style.display = 'block';
      if (isPending) {
        el.style.background = '#fef3c7';
        el.style.color = '#92400e';
        el.style.border = '1px solid #f59e0b';
      } else if (isError) {
        el.style.background = '#fee2e2';
        el.style.color = '#b91c1c';
        el.style.border = '1px solid #f87171';
      } else {
        el.style.background = '#dcfce7';
        el.style.color = '#15803d';
        el.style.border = '1px solid #86efac';
      }
      el.textContent = msg;
    }

    function dismissBackupReminder() {
      backupReminderDismissed = true;
      const banner = document.getElementById('backup-reminder-banner');
      if (banner) banner.style.display = 'none';
    }

    async function loadBackupStatus() {
      try {
        const res = await fetch('/api/system/backup/status');
        const data = await res.json();
        if (!data.success || !data.data) return;
        const info = data.data;

        const dirInput = document.getElementById('backup-dir-input');
        if (dirInput) dirInput.value = info.backup_dir || '';
        const cycleSel = document.getElementById('backup-reminder-cycle');
        if (cycleSel) cycleSel.value = info.reminder_cycle || 'weekly';

        const lastInfo = document.getElementById('backup-last-info');
        if (lastInfo) {
          if (info.last_backup_display) {
            const daysTxt = info.days_since_last_backup !== null ? `（距今 ${info.days_since_last_backup} 天）` : '';
            lastInfo.textContent = `上次備份時間：${info.last_backup_display} ${daysTxt}`;
          } else {
            lastInfo.textContent = '上次備份時間：尚未備份過';
          }
        }

        // 更新頂部定期備份提醒橫幅
        const banner = document.getElementById('backup-reminder-banner');
        const bannerText = document.getElementById('backup-reminder-text');
        if (banner && bannerText) {
          if (info.reminder_due && !backupReminderDismissed) {
            const cycleLabel = info.reminder_cycle === 'monthly' ? '每月' : '每週';
            if (info.last_backup_display) {
              bannerText.textContent = `⚠️ ${cycleLabel}備份提醒：距離上次資料庫備份（${info.last_backup_display}）已達 ${info.days_since_last_backup} 天，建議立即備份！`;
            } else {
              bannerText.textContent = `⚠️ ${cycleLabel}備份提醒：系統尚未建立過資料庫備份，建議立即備份以保障資料安全！`;
            }
            banner.style.display = 'block';
          } else {
            banner.style.display = 'none';
          }
        }

        // 渲染備份檔案清單
        const tbody = document.querySelector('#backup-files-table tbody');
        if (tbody) {
          const files = info.files || [];
          if (files.length === 0) {
            tbody.innerHTML = '<tr><td colspan="4" style="text-align: center; color: var(--text-muted); padding: 1.25rem;">此備份目錄尚無備份檔案</td></tr>';
          } else {
            tbody.innerHTML = files.map(f => {
              const badge = f.is_pre_restore
                ? '<span class="badge badge-warning" style="margin-left: 0.35rem;">還原前快照</span>'
                : '';
              const safeName = escapeHtml(f.filename);
              const jsName = JSON.stringify(f.filename);
              return `
                <tr>
                  <td><code>${safeName}</code>${badge}</td>
                  <td style="font-size: 0.85rem; color: var(--text-muted);">${escapeHtml(f.modified_at)}</td>
                  <td style="font-size: 0.85rem;">${formatFileSize(f.size_bytes)}</td>
                  <td style="white-space: nowrap;">
                    <button type="button" class="btn btn-outline btn-sm" style="color: var(--primary); border-color: var(--primary);" onclick='restoreBackupFile(${jsName})'>🔄 還原</button>
                    <a class="btn btn-outline btn-sm" style="text-decoration: none;" href="/api/system/backup/download/${encodeURIComponent(f.filename)}" download>⬇️ 下載</a>
                    <button type="button" class="btn btn-outline btn-sm" style="color: var(--danger); border-color: var(--danger);" onclick='deleteBackupFile(${jsName})'>🗑️ 刪除</button>
                  </td>
                </tr>
              `;
            }).join('');
          }
        }
      } catch (e) {
        console.error('載入備份狀態失敗:', e);
      }
    }

    async function pickBackupDir() {
      showBackupStatusMsg('⏳ 正在開啟本機資料夾選擇視窗，請在彈出視窗中選擇目錄...', false, true);
      try {
        const res = await fetch('/api/system/backup/pick-dir', { method: 'POST' });
        const data = await res.json();
        if (data.success) {
          if (data.data) {
            const dirInput = document.getElementById('backup-dir-input');
            if (dirInput) dirInput.value = data.data;
            showBackupStatusMsg(`已選擇目錄：${data.data}（請點擊「儲存備份設定」或直接「立即備份」）`);
          } else {
            const el = document.getElementById('backup-action-status');
            if (el) el.style.display = 'none';
          }
        } else {
          showBackupStatusMsg(data.message || '選擇資料夾失敗', true);
        }
      } catch (e) {
        showBackupStatusMsg('呼叫資料夾選擇視窗失敗：' + e.message, true);
      }
    }

    async function saveBackupConfig() {
      const backupDir = document.getElementById('backup-dir-input')?.value.trim() || '';
      const reminderCycle = document.getElementById('backup-reminder-cycle')?.value || 'weekly';
      try {
        const res = await fetch('/api/system/backup/config', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ backup_dir: backupDir, reminder_cycle: reminderCycle }),
        });
        const data = await res.json();
        if (data.success) {
          backupReminderDismissed = false;
          showBackupStatusMsg(`✅ ${data.message}`);
          loadBackupStatus();
        } else {
          showBackupStatusMsg(`❌ ${data.message}`, true);
        }
      } catch (e) {
        showBackupStatusMsg('儲存設定失敗：' + e.message, true);
      }
    }

    async function createDatabaseBackup(fromBanner = false) {
      const btnSettings = document.getElementById('btn-create-backup');
      const btnBanner = document.getElementById('btn-banner-backup-now');
      if (btnSettings) { btnSettings.disabled = true; btnSettings.textContent = '⏳ 備份中...'; }
      if (btnBanner) { btnBanner.disabled = true; btnBanner.textContent = '⏳ 備份中...'; }
      if (!fromBanner) {
        showBackupStatusMsg('⏳ 正在建立完整資料庫備份檔，請稍候...', false, true);
      }

      const customDir = !fromBanner ? (document.getElementById('backup-dir-input')?.value.trim() || null) : null;
      try {
        const res = await fetch('/api/system/backup/create', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ backup_dir: customDir }),
        });
        const data = await res.json();
        if (data.success) {
          backupReminderDismissed = false;
          showBackupStatusMsg(`✅ ${data.message}`);
          if (fromBanner) alert(data.message);
          loadBackupStatus();
        } else {
          showBackupStatusMsg(`❌ ${data.message}`, true);
          if (fromBanner) alert(data.message);
        }
      } catch (e) {
        const msg = '執行備份失敗：' + e.message;
        showBackupStatusMsg(`❌ ${msg}`, true);
        if (fromBanner) alert(msg);
      } finally {
        if (btnSettings) { btnSettings.disabled = false; btnSettings.textContent = '⚡ 立即備份資料庫'; }
        if (btnBanner) { btnBanner.disabled = false; btnBanner.textContent = '💾 立即備份'; }
      }
    }

    async function deleteBackupFile(filename) {
      if (!confirm(`確定要刪除備份檔「${filename}」嗎？此動作無法復原。`)) return;
      try {
        const res = await fetch(`/api/system/backup/${encodeURIComponent(filename)}`, { method: 'DELETE' });
        const data = await res.json();
        if (data.success) {
          showBackupStatusMsg(`✅ ${data.message}`);
          loadBackupStatus();
        } else {
          showBackupStatusMsg(`❌ ${data.message}`, true);
        }
      } catch (e) {
        showBackupStatusMsg('刪除備份檔失敗：' + e.message, true);
      }
    }

    async function restoreBackupFile(filename) {
      const warning = [
        `確定要從備份檔「${filename}」還原資料庫嗎？`,
        '',
        '⚠️ 重要說明：',
        '1. 目前的書籍、讀者、借閱紀錄與身分別資料將被覆蓋為該備份當時的資料。',
        '2. 【管理員帳號與密碼會保留目前最新版本】，不會還原成舊備份中的帳號密碼。',
        '3. 還原前系統會自動先備份目前資料庫（library_before_restore_*.db）以策安全。',
        '4. 若該備份為舊版本格式，還原後系統會自動升級資料表結構。',
      ].join('\n');
      if (!confirm(warning)) return;

      showBackupStatusMsg(`⏳ 正在驗證並從 ${filename} 還原資料庫，請耐心等待，切勿關閉視窗...`, false, true);
      try {
        const res = await fetch('/api/system/backup/restore', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ filename }),
        });
        const data = await res.json();
        if (data.success) {
          showBackupStatusMsg(`✅ ${data.message}`);
          alert(data.message + '\n\n點擊確定後將自動重新載入頁面。');
          window.location.reload();
        } else {
          showBackupStatusMsg(`❌ ${data.message}`, true);
          alert(data.message);
        }
      } catch (e) {
        const msg = '還原資料庫失敗：' + e.message;
        showBackupStatusMsg(`❌ ${msg}`, true);
        alert(msg);
      }
    }

    async function uploadRestoreDbFile(input) {
      const file = input.files && input.files[0];
      if (!file) return;
      const warning = [
        `確定要從本機檔案「${file.name}」還原資料庫嗎？`,
        '',
        '⚠️ 重要說明：',
        '1. 目前的書籍、讀者、借閱紀錄與身分別資料將被覆蓋為該備份當時的資料。',
        '2. 【管理員帳號與密碼會保留目前最新版本】，不會還原成舊備份中的帳號密碼。',
        '3. 還原前系統會自動先備份目前資料庫（library_before_restore_*.db）以策安全。',
        '4. 若該備份為舊版本格式，還原後系統會自動升級資料表結構。',
      ].join('\n');
      if (!confirm(warning)) {
        input.value = '';
        return;
      }

      showBackupStatusMsg(`⏳ 正在上傳並還原 ${file.name}，請耐心等待，切勿關閉視窗...`, false, true);
      const formData = new FormData();
      formData.append('file', file);
      try {
        const res = await fetch('/api/system/backup/restore-upload', {
          method: 'POST',
          body: formData,
        });
        const data = await res.json();
        if (data.success) {
          showBackupStatusMsg(`✅ ${data.message}`);
          alert(data.message + '\n\n點擊確定後將自動重新載入頁面。');
          window.location.reload();
        } else {
          showBackupStatusMsg(`❌ ${data.message}`, true);
          alert(data.message);
        }
      } catch (e) {
        const msg = '上傳還原失敗：' + e.message;
        showBackupStatusMsg(`❌ ${msg}`, true);
        alert(msg);
      } finally {
        input.value = '';
      }
    }

    // 初始化載入
    window.addEventListener('DOMContentLoaded', () => {
      checkAuthStatus();
      checkForUpdates();
      loadBackupStatus();
      setInterval(loadBackupStatus, 30 * 60 * 1000);
    });
  </script>
</body>
</html>
"##;
