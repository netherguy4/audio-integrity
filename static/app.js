const dictionary = {
  ru: {
    offline:'Связь потеряна. Данные обновятся после переподключения.',loginTitle:'Войти в Audio Integrity',loginLead:'Проверяйте аудиофайлы до и после импорта — без изменений в медиатеке.',username:'Логин',password:'Пароль',loginError:'Неверный логин или пароль.',loginAction:'Войти',readOnly:'Медиатека без изменений',overview:'Состояние',results:'Файлы',history:'История',lidarrPolicy:'Защита импорта',failClosed:'Непроверенные файлы блокируются',logout:'Выйти',pageTitle:'Состояние медиатеки',language:'Язык',idle:'Можно запускать',discovering:'Собираем список файлов',scanning:'Проверяем медиатеку',cancelling:'Завершаем текущий файл…',completed:'Проверка завершена',cancelled:'Проверка остановлена',failed:'Не удалось завершить проверку',scanTitle:'Проверка медиатеки',scanLead:'Проверим новые и изменённые файлы. Для остальных используем сохранённые результаты.',scanIncremental:'Проверить изменения',scanFull:'Проверить всё заново',cancel:'Остановить проверку',fullConfirmTitle:'Проверить все файлы заново?',fullConfirmText:'Сохранённые результаты не используются. Проверка прочитает около 786 GiB и займёт несколько часов.',startFull:'Проверить всё заново',nevermind:'Отменить',currentFile:'Сейчас проверяется',throughput:'Скорость чтения',eta:'Осталось примерно',connection:'Обновление данных',connecting:'Подключаемся…',live:'Данные актуальны',reconnecting:'Восстанавливаем связь…',liveLog:'Ход проверки',liveLogHint:'обновляется в реальном времени',noEvents:'Здесь появится ход проверки.',verified:'Без ошибок',cache:'Из сохранённых',corrupt:'Не прошли проверку',suspect:'Возможно пережаты',read:'Прочитано',tracks:'файлов проверено',lastAudit:'последняя полная проверка',healthy:'Без ошибок',resultTitle:'Проверенные файлы',authNote:'«Возможно пережат» означает, что спектральный анализ обнаружил признаки перекодирования из MP3/AAC. Это не означает, что файл повреждён.',allResults:'Все файлы',onlyCorrupt:'Не прошли проверку',onlySuspect:'Возможно пережаты',onlyErrors:'Не удалось проверить',onlyHealthy:'Без ошибок',search:'Найти файл…',state:'Результат',path:'Файл',authenticity:'Качество источника',format:'Формат',checked:'Дата проверки',noResults:'Ещё нет проверенных файлов',noResultsLead:'Запустите проверку, чтобы увидеть состояние медиатеки.',footerReadOnly:'Audio Integrity не изменяет и не удаляет файлы.',healthyBadge:'Без ошибок',corruptBadge:'Ошибка декодирования',errorBadge:'Не проверен',likely_genuine:'Похоже на lossless',likely_lossy:'Возможно пережат',unknown:'Нет данных',pending:'Анализируем',not_applicable:'Не анализируется',details:'Показать технические детали',incremental:'Изменения',full:'Вся медиатека',files:'файлов',cached:'без повтора',defects:'не прошли',never:'ещё не было'
  },
  en: {
    offline:'Connection lost. Data will refresh after reconnection.',loginTitle:'Sign in to Audio Integrity',loginLead:'Check audio before and after import without changing your library.',username:'Username',password:'Password',loginError:'Incorrect username or password.',loginAction:'Sign in',readOnly:'Library stays unchanged',overview:'Status',results:'Files',history:'History',lidarrPolicy:'Import protection',failClosed:'Unverified files are blocked',logout:'Sign out',pageTitle:'Library status',language:'Language',idle:'Ready to check',discovering:'Finding audio files',scanning:'Checking library',cancelling:'Finishing the current file…',completed:'Check completed',cancelled:'Check stopped',failed:'Could not complete check',scanTitle:'Library check',scanLead:'New and changed files will be checked. Saved results are used for everything else.',scanIncremental:'Check changes',scanFull:'Recheck everything',cancel:'Stop check',fullConfirmTitle:'Recheck every file?',fullConfirmText:'Saved results will not be used. Reading about 786 GiB can take several hours.',startFull:'Recheck everything',nevermind:'Cancel',currentFile:'Checking now',throughput:'Read speed',eta:'About',connection:'Data updates',connecting:'Connecting…',live:'Up to date',reconnecting:'Restoring connection…',liveLog:'Check activity',liveLogHint:'updates in real time',noEvents:'Check activity will appear here.',verified:'No errors',cache:'From saved results',corrupt:'Failed check',suspect:'Possibly transcoded',read:'Read',tracks:'files checked',lastAudit:'last full check',healthy:'No errors',resultTitle:'Checked files',authNote:'“Possibly transcoded” means spectral analysis found signs of an MP3/AAC source. It does not mean the file is damaged.',allResults:'All files',onlyCorrupt:'Failed check',onlySuspect:'Possibly transcoded',onlyErrors:'Could not be checked',onlyHealthy:'No errors',search:'Find a file…',state:'Result',path:'File',authenticity:'Source quality',format:'Format',checked:'Checked at',noResults:'No files checked yet',noResultsLead:'Run a check to see the health of your library.',footerReadOnly:'Audio Integrity never changes or deletes your files.',healthyBadge:'No errors',corruptBadge:'Decode error',errorBadge:'Not checked',likely_genuine:'Appears lossless',likely_lossy:'Possibly transcoded',unknown:'No data',pending:'Analyzing',not_applicable:'Not analyzed',details:'Show technical details',incremental:'Changes',full:'Entire library',files:'files',cached:'not rechecked',defects:'failed',never:'not yet'
  }
};

const supplementaryCopy = {
  ru: { running:'Идёт сейчас',previous:'Назад',next:'Вперёд',perPage:'На странице',paginationLabel:'Навигация по файлам' },
  en: { running:'In progress',previous:'Previous',next:'Next',perPage:'Per page',paginationLabel:'File navigation' }
};

const savedPageSize = Number(localStorage.getItem('audio-integrity-page-size'));
const state = { lang: localStorage.getItem('audio-integrity-lang') || (navigator.language.startsWith('ru') ? 'ru' : 'en'), socket:null, reconnectTimer:null, connectionKey:'connecting', lastCounters:null, scanStatus:null, historyRuns:[], resultsPage:0, resultsPageSize:[25,50,100].includes(savedPageSize) ? savedPageSize : 25, resultsRequestId:0 };
const $ = selector => document.querySelector(selector);
const $$ = selector => [...document.querySelectorAll(selector)];
const t = key => dictionary[state.lang][key] || supplementaryCopy[state.lang][key] || key;

function applyLanguage() {
  document.documentElement.lang = state.lang;
  $('#language').value = state.lang;
  $$('[data-i18n]').forEach(node => { node.textContent = t(node.dataset.i18n); });
  $$('[data-i18n-placeholder]').forEach(node => { node.placeholder = t(node.dataset.i18nPlaceholder); });
  $('#results-pagination').setAttribute('aria-label', t('paginationLabel'));
  localStorage.setItem('audio-integrity-lang', state.lang);
  refreshAll();
}

function applyTheme(theme) {
  document.documentElement.dataset.theme = theme;
  localStorage.setItem('audio-integrity-theme', theme);
}

async function api(path, options = {}) {
  const response = await fetch(path, { credentials:'same-origin', headers:{'Content-Type':'application/json', ...(options.headers || {})}, ...options });
  if (response.status === 401 && path !== '/api/login') showLogin();
  if (!response.ok) throw new Error(await response.text() || response.statusText);
  $('#offline').hidden = true;
  return response.status === 204 ? null : response.json();
}

function showLogin() {
  $('#app-view').hidden = true;
  $('#login-view').hidden = false;
  clearTimeout(state.reconnectTimer);
  if (state.socket) state.socket.close();
  state.socket = null;
}

function showApp() {
  $('#login-view').hidden = true;
  $('#app-view').hidden = false;
  refreshAll();
  connectRealtime();
}

function formatNumber(value) { return new Intl.NumberFormat(state.lang).format(value || 0); }
function formatBytes(value) {
  if (!value) return '0 B';
  const units = ['B','KiB','MiB','GiB','TiB'];
  const index = Math.min(Math.floor(Math.log(value) / Math.log(1024)), units.length - 1);
  return `${(value / 1024 ** index).toLocaleString(state.lang, {maximumFractionDigits:index > 2 ? 2 : 1})} ${units[index]}`;
}
function formatDate(value) { return value ? new Intl.DateTimeFormat(state.lang, {dateStyle:'medium',timeStyle:'short'}).format(new Date(value)) : t('never'); }
function formatRange(start, end, total) { return state.lang === 'ru' ? `${formatNumber(start)}–${formatNumber(end)} из ${formatNumber(total)} файлов` : `${formatNumber(start)}–${formatNumber(end)} of ${formatNumber(total)} files`; }
function formatPage(page, total) { return state.lang === 'ru' ? `Страница ${formatNumber(page)} из ${formatNumber(total)}` : `Page ${formatNumber(page)} of ${formatNumber(total)}`; }
function formatDuration(seconds) {
  if (!Number.isFinite(seconds) || seconds < 0) return '—';
  if (seconds < 60) return `< 1 ${state.lang === 'ru' ? 'мин' : 'min'}`;
  const hours = Math.floor(seconds / 3600);
  const minutes = Math.ceil((seconds % 3600) / 60);
  return hours ? `${hours} ${state.lang === 'ru' ? 'ч' : 'h'} ${minutes} ${state.lang === 'ru' ? 'мин' : 'min'}` : `${minutes} ${state.lang === 'ru' ? 'мин' : 'min'}`;
}
function escapeHtml(value) { const div=document.createElement('div'); div.textContent=value ?? ''; return div.innerHTML; }

async function refreshAll() {
  if ($('#app-view').hidden) return;
  await Promise.allSettled([refreshStatus(), refreshSummary(), refreshResults(), refreshHistory()]);
}

async function refreshStatus() {
  if ($('#app-view').hidden) return;
  try {
    const scan = await api('/api/status');
    renderStatus(scan);
  } catch { $('#offline').hidden = false; }
}

function renderStatus(scan) {
    state.scanStatus = scan;
    const running = ['discovering','scanning','cancelling'].includes(scan.phase);
    $('#phase').textContent = t(scan.phase);
    $('#phase-dot').className = `status-dot ${running ? 'running' : scan.phase === 'failed' ? 'failed' : scan.phase === 'completed' ? 'ok' : 'idle'}`;
    const percent = scan.totalBytes ? Math.min(100, scan.processedBytes / scan.totalBytes * 100) : 0;
    $('#progress-label').textContent = `${formatNumber(scan.processedFiles)} / ${formatNumber(scan.totalFiles)}`;
    $('#progress-percent').textContent = `${percent.toFixed(percent < 10 ? 1 : 0)}%`;
    $('#progress-bar').style.width = `${percent}%`;
    $('#scan-activity').hidden = !running;
    $('#current-path').textContent = scan.currentPath || '—';
    $('#current-path').title = scan.currentPath || '';
    $('#current-validator').textContent = scan.currentValidator ? `${scan.currentValidator}${scan.currentValidator.startsWith('flac') ? ' · isflac spectrum' : ''}` : 'flac --test · isflac spectrum';
    $('#metric-verified').textContent = formatNumber(scan.verifiedFiles);
    $('#metric-skipped').textContent = formatNumber(scan.skippedFiles);
    $('#metric-corrupt').textContent = formatNumber(scan.corruptFiles);
    $('#metric-suspect').textContent = formatNumber(scan.suspectFiles);
    $('#metric-bytes').textContent = formatBytes(scan.readBytes);
    const elapsed = scan.startedAt ? Math.max(1, (Date.now() - Date.parse(scan.startedAt)) / 1000) : 0;
    const throughput = elapsed && scan.readBytes ? scan.readBytes / elapsed : 0;
    const freshlyChecked = scan.verifiedFiles + scan.corruptFiles + scan.errorFiles;
    const filesPerSecond = elapsed && freshlyChecked ? freshlyChecked / elapsed : 0;
    const eta = running && filesPerSecond ? (scan.totalFiles - scan.processedFiles) / filesPerSecond : NaN;
    $('#live-throughput').textContent = throughput ? `${formatBytes(throughput)}/s` : '—';
    $('#live-eta').textContent = formatDuration(eta);
    const log = scan.log || [];
    $('#event-log-list').innerHTML = log.length ? log.slice(-6).map(line => `<li class="${escapeHtml(line.level)}"><time>${escapeHtml(new Intl.DateTimeFormat(state.lang,{hour:'2-digit',minute:'2-digit',second:'2-digit'}).format(new Date(line.at)))}</time><span>${escapeHtml(line.text)}</span></li>`).join('') : `<li class="empty-log">${t('noEvents')}</li>`;
    $('#scan-incremental').disabled = running;
    $('#scan-full').disabled = running;
    $('#scan-cancel').hidden = !running;
    const counters = `${scan.corruptFiles}:${scan.suspectFiles}:${scan.errorFiles}`;
    if (state.lastCounters !== null && state.lastCounters !== counters) {
      Promise.allSettled([refreshSummary(), refreshResults()]);
    }
    state.lastCounters = counters;
    renderHistory(state.historyRuns);
    if (!running && ['completed','cancelled','failed'].includes(scan.phase)) refreshHistory();
}

function setConnection(key) {
  state.connectionKey = key;
  $('#live-connection').dataset.i18n = key;
  $('#live-connection').textContent = t(key);
  $('#live-connection').className = key;
}

function connectRealtime() {
  if ($('#app-view').hidden || state.socket?.readyState === WebSocket.OPEN || state.socket?.readyState === WebSocket.CONNECTING) return;
  clearTimeout(state.reconnectTimer);
  setConnection('connecting');
  const protocol = location.protocol === 'https:' ? 'wss:' : 'ws:';
  const socket = new WebSocket(`${protocol}//${location.host}/api/realtime`);
  state.socket = socket;
  socket.addEventListener('open', () => { setConnection('live'); $('#offline').hidden = true; });
  socket.addEventListener('message', event => {
    try {
      const payload = JSON.parse(event.data);
      if (payload.kind === 'status') renderStatus(payload.status);
    } catch {}
  });
  socket.addEventListener('close', () => {
    if (state.socket === socket) state.socket = null;
    if (!$('#app-view').hidden) {
      setConnection('reconnecting');
      state.reconnectTimer = setTimeout(connectRealtime, 1500);
    }
  });
  socket.addEventListener('error', () => socket.close());
}

async function refreshSummary() {
  try {
    const summary = await api('/api/summary');
    $('#known-files').textContent = formatNumber(summary.knownFiles);
    $('#known-bytes').textContent = formatBytes(summary.checkedBytes);
    $('#last-audit').textContent = formatDate(summary.lastCompletedAt);
    $('#summary-healthy').textContent = formatNumber(summary.healthyFiles);
    $('#summary-corrupt').textContent = formatNumber(summary.corruptFiles);
    $('#summary-suspect').textContent = formatNumber(summary.suspectFiles);
    $('#validator-version').textContent = summary.validatorVersion;
  } catch {}
}

function badge(value, label = null) { return `<span class="badge ${escapeHtml(value)}">${escapeHtml(label || t(value))}</span>`; }

async function refreshResults() {
  const requestId = ++state.resultsRequestId;
  const query = new URLSearchParams({verdict:$('#verdict-filter').value, query:$('#search').value, limit:String(state.resultsPageSize), offset:String(state.resultsPage * state.resultsPageSize)});
  try {
    const page = await api(`/api/results?${query}`);
    if (requestId !== state.resultsRequestId) return;
    const totalPages = Math.max(1, Math.ceil(page.total / page.limit));
    if (page.total && state.resultsPage >= totalPages) {
      state.resultsPage = totalPages - 1;
      return refreshResults();
    }
    const rows = page.items;
    $('#results-empty').hidden = rows.length > 0;
    $('#results-body').innerHTML = rows.map(row => `<tr>
      <td>${badge(row.verdict, t(`${row.verdict}Badge`))}</td>
      <td class="path-cell"><code>${escapeHtml(row.path)}</code><details><summary>${t('details')}</summary>${escapeHtml(row.message)}</details></td>
      <td>${badge(row.authenticity)} </td>
      <td>${escapeHtml(row.format.toUpperCase())}<br><small class="muted">${formatBytes(row.size)}</small></td>
      <td>${escapeHtml(formatDate(row.checkedAt))}<br><small class="muted">${formatNumber(row.durationMs)} ms</small></td>
    </tr>`).join('');
    $('#results-pagination').hidden = page.total === 0;
    if (page.total) {
      const start = page.offset + 1;
      const end = Math.min(page.offset + rows.length, page.total);
      $('#results-range').textContent = formatRange(start, end, page.total);
      $('#results-page').textContent = formatPage(state.resultsPage + 1, totalPages);
      $('#results-prev').disabled = state.resultsPage === 0;
      $('#results-next').disabled = state.resultsPage + 1 >= totalPages;
    }
  } catch {}
}

function renderHistory(rows) {
  $('#history-list').innerHTML = rows.length ? rows.map(run => {
      const live = run.status === 'running' && state.scanStatus?.runId === run.id ? state.scanStatus : null;
      const totalFiles = live?.totalFiles ?? run.totalFiles;
      const skippedFiles = live?.skippedFiles ?? run.skippedFiles;
      const corruptFiles = live?.corruptFiles ?? run.corruptFiles;
      return `<div class="history-row">
      <span>${badge(run.status)}</span>
      <span class="run-meta"><strong>${t(run.mode)}</strong><small>${formatDate(run.startedAt)}</small></span>
      <span class="number"><strong>${formatNumber(totalFiles)}</strong><br><small>${t('files')}</small></span>
      <span class="number"><strong>${formatNumber(skippedFiles)}</strong><br><small>${t('cached')}</small></span>
      <span class="number"><strong>${formatNumber(corruptFiles)}</strong><br><small>${t('defects')}</small></span>
    </div>`;
  }).join('') : `<div class="history-empty">${t('noResultsLead')}</div>`;
}

async function refreshHistory() {
  try {
    state.historyRuns = await api('/api/history');
    renderHistory(state.historyRuns);
  } catch {}
}

async function startScan(mode) {
  $('#full-confirm').hidden = true;
  try { await api('/api/scans', {method:'POST', body:JSON.stringify({mode})}); await refreshStatus(); }
  catch (error) { $('#offline').textContent = error.message; $('#offline').hidden = false; }
}

$('#login-form').addEventListener('submit', async event => {
  event.preventDefault();
  const data = new FormData(event.currentTarget);
  try { await api('/api/login', {method:'POST',body:JSON.stringify(Object.fromEntries(data))}); $('#login-error').hidden=true; showApp(); }
  catch { $('#login-error').hidden=false; }
});
$('#logout').addEventListener('click', async () => { await api('/api/logout',{method:'POST'}).catch(()=>{}); showLogin(); });
$('#language').addEventListener('change', event => { state.lang=event.target.value; applyLanguage(); });
$('#theme').addEventListener('click', () => applyTheme(document.documentElement.dataset.theme === 'dark' ? 'light' : 'dark'));
$('#scan-incremental').addEventListener('click', () => startScan('incremental'));
$('#scan-full').addEventListener('click', () => { $('#full-confirm').hidden=false; });
$('#full-confirm-no').addEventListener('click', () => { $('#full-confirm').hidden=true; });
$('#full-confirm-yes').addEventListener('click', () => startScan('full'));
$('#scan-cancel').addEventListener('click', () => api('/api/scans/cancel',{method:'POST'}).then(refreshStatus).catch(()=>{}));
$('#verdict-filter').addEventListener('change', () => { state.resultsPage=0; refreshResults(); });
$('#search').addEventListener('input', () => { state.resultsPage=0; clearTimeout(state.searchTimer); state.searchTimer=setTimeout(refreshResults,250); });
$('#results-prev').addEventListener('click', () => { if (state.resultsPage > 0) { state.resultsPage--; refreshResults(); } });
$('#results-next').addEventListener('click', () => { state.resultsPage++; refreshResults(); });
$('#results-page-size').value = String(state.resultsPageSize);
$('#results-page-size').addEventListener('change', event => { state.resultsPageSize=Number(event.target.value); state.resultsPage=0; localStorage.setItem('audio-integrity-page-size', String(state.resultsPageSize)); refreshResults(); });
$$('.nav-link[href]').forEach(link => link.addEventListener('click', () => { $$('.nav-link').forEach(item => item.classList.remove('active')); link.classList.add('active'); }));

applyTheme(localStorage.getItem('audio-integrity-theme') || (matchMedia('(prefers-color-scheme:dark)').matches ? 'dark' : 'light'));
applyLanguage();
api('/api/session').then(session => session.authenticated ? showApp() : showLogin()).catch(showLogin);
