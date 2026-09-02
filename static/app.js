const dictionary = {
  ru: {
    offline:'Нет связи с сервером. Повторяем подключение…',loginTitle:'Вход в консоль',loginLead:'Строгая проверка библиотеки без изменения файлов.',username:'Пользователь',password:'Пароль',loginError:'Не удалось войти.',loginAction:'Войти',readOnly:'Только чтение',overview:'Обзор',results:'Результаты',history:'История',lidarrGate:'Защита Lidarr',failClosed:'fail-closed',logout:'Выйти',libraryControl:'Контроль медиатеки',pageTitle:'Целостность аудио',language:'Язык',idle:'Готово к проверке',discovering:'Поиск файлов',scanning:'Идёт проверка',cancelling:'Останавливаем',completed:'Проверка завершена',cancelled:'Проверка остановлена',failed:'Ошибка проверки',scanTitle:'Проверка библиотеки',scanLead:'Неизменённые файлы будут взяты из кэша. Музыка открывается только для чтения.',scanIncremental:'Проверить изменения',scanFull:'Полный аудит',cancel:'Остановить',fullConfirmTitle:'Перепроверить всю библиотеку?',fullConfirmText:'Кэш будет проигнорирован; чтение ~786 GiB может занять несколько часов.',startFull:'Начать полный аудит',nevermind:'Не сейчас',currentFile:'Текущий файл',verified:'Проверено',cache:'Из кэша',corrupt:'Повреждено',suspect:'Вероятно lossy',read:'Прочитано',knownState:'Известное состояние',tracks:'треков',lastAudit:'последний аудит',healthy:'Целые',evidence:'Проверяемые данные',resultTitle:'Результаты файлов',allResults:'Все результаты',onlyCorrupt:'Только повреждённые',onlySuspect:'Вероятно lossy',onlyErrors:'Ошибки проверки',onlyHealthy:'Только целые',search:'Поиск по пути…',state:'Состояние',path:'Путь',authenticity:'Источник',format:'Формат',checked:'Проверен',noResults:'Пока нет результатов',noResultsLead:'Запустите первую проверку библиотеки.',auditTrail:'Журнал аудита',footerReadOnly:'Файловая система медиатеки подключена только для чтения.',healthyBadge:'Целый',corruptBadge:'Повреждён',errorBadge:'Ошибка',likely_genuine:'Вероятно lossless',likely_lossy:'Вероятно lossy',unknown:'Неизвестно',not_applicable:'Не применимо',details:'Детали валидатора',incremental:'Изменения',full:'Полный',files:'файлов',cached:'кэш',defects:'дефектов',never:'никогда'
  },
  en: {
    offline:'Server unavailable. Reconnecting…',loginTitle:'Console sign in',loginLead:'Strict library validation without modifying files.',username:'Username',password:'Password',loginError:'Sign in failed.',loginAction:'Sign in',readOnly:'Read only',overview:'Overview',results:'Results',history:'History',lidarrGate:'Lidarr gate',failClosed:'fail-closed',logout:'Sign out',libraryControl:'Library control',pageTitle:'Audio integrity',language:'Language',idle:'Ready to scan',discovering:'Discovering files',scanning:'Scan running',cancelling:'Stopping safely',completed:'Scan completed',cancelled:'Scan stopped',failed:'Scan failed',scanTitle:'Library verification',scanLead:'Unchanged files will use cached evidence. Music is opened read-only.',scanIncremental:'Check changes',scanFull:'Full audit',cancel:'Stop',fullConfirmTitle:'Recheck the entire library?',fullConfirmText:'The cache will be ignored; reading ~786 GiB can take several hours.',startFull:'Start full audit',nevermind:'Not now',currentFile:'Current file',verified:'Verified',cache:'From cache',corrupt:'Corrupt',suspect:'Likely lossy',read:'Read',knownState:'Known state',tracks:'tracks',lastAudit:'last audit',healthy:'Healthy',evidence:'Verifiable evidence',resultTitle:'File results',allResults:'All results',onlyCorrupt:'Corrupt only',onlySuspect:'Likely lossy',onlyErrors:'Validation errors',onlyHealthy:'Healthy only',search:'Search paths…',state:'State',path:'Path',authenticity:'Source',format:'Format',checked:'Checked',noResults:'No results yet',noResultsLead:'Start the first library scan.',auditTrail:'Audit trail',footerReadOnly:'The library filesystem is mounted read-only.',healthyBadge:'Healthy',corruptBadge:'Corrupt',errorBadge:'Error',likely_genuine:'Likely lossless',likely_lossy:'Likely lossy',unknown:'Unknown',not_applicable:'Not applicable',details:'Validator details',incremental:'Changes',full:'Full',files:'files',cached:'cached',defects:'defects',never:'never'
  }
};

const state = { lang: localStorage.getItem('audio-integrity-lang') || (navigator.language.startsWith('ru') ? 'ru' : 'en'), timer: null, resultsTimer: null };
const $ = selector => document.querySelector(selector);
const $$ = selector => [...document.querySelectorAll(selector)];
const t = key => dictionary[state.lang][key] || key;

function applyLanguage() {
  document.documentElement.lang = state.lang;
  $('#language').value = state.lang;
  $$('[data-i18n]').forEach(node => { node.textContent = t(node.dataset.i18n); });
  $$('[data-i18n-placeholder]').forEach(node => { node.placeholder = t(node.dataset.i18nPlaceholder); });
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
  clearInterval(state.timer);
}

function showApp() {
  $('#login-view').hidden = true;
  $('#app-view').hidden = false;
  refreshAll();
  clearInterval(state.timer);
  state.timer = setInterval(refreshStatus, 1600);
}

function formatNumber(value) { return new Intl.NumberFormat(state.lang).format(value || 0); }
function formatBytes(value) {
  if (!value) return '0 B';
  const units = ['B','KiB','MiB','GiB','TiB'];
  const index = Math.min(Math.floor(Math.log(value) / Math.log(1024)), units.length - 1);
  return `${(value / 1024 ** index).toLocaleString(state.lang, {maximumFractionDigits:index > 2 ? 2 : 1})} ${units[index]}`;
}
function formatDate(value) { return value ? new Intl.DateTimeFormat(state.lang, {dateStyle:'medium',timeStyle:'short'}).format(new Date(value)) : t('never'); }
function escapeHtml(value) { const div=document.createElement('div'); div.textContent=value ?? ''; return div.innerHTML; }

async function refreshAll() {
  if ($('#app-view').hidden) return;
  await Promise.allSettled([refreshStatus(), refreshSummary(), refreshResults(), refreshHistory()]);
}

async function refreshStatus() {
  if ($('#app-view').hidden) return;
  try {
    const scan = await api('/api/status');
    const running = ['discovering','scanning','cancelling'].includes(scan.phase);
    $('#phase').textContent = t(scan.phase);
    $('#phase-dot').className = `status-dot ${running ? 'running' : scan.phase === 'failed' ? 'failed' : scan.phase === 'completed' ? 'ok' : 'idle'}`;
    const percent = scan.totalBytes ? Math.min(100, scan.processedBytes / scan.totalBytes * 100) : 0;
    $('#progress-label').textContent = `${formatNumber(scan.processedFiles)} / ${formatNumber(scan.totalFiles)}`;
    $('#progress-percent').textContent = `${percent.toFixed(percent < 10 ? 1 : 0)}%`;
    $('#progress-bar').style.width = `${percent}%`;
    $('#current-path').textContent = scan.currentPath || '—';
    $('#current-path').title = scan.currentPath || '';
    $('#current-validator').textContent = scan.currentValidator ? `${scan.currentValidator}${scan.currentValidator.startsWith('flac') ? ' · isflac spectrum' : ''}` : 'flac --test · isflac spectrum';
    $('#metric-verified').textContent = formatNumber(scan.verifiedFiles);
    $('#metric-skipped').textContent = formatNumber(scan.skippedFiles);
    $('#metric-corrupt').textContent = formatNumber(scan.corruptFiles);
    $('#metric-suspect').textContent = formatNumber(scan.suspectFiles);
    $('#metric-bytes').textContent = formatBytes(scan.processedBytes);
    $('#scan-incremental').disabled = running;
    $('#scan-full').disabled = running;
    $('#scan-cancel').hidden = !running;
    if (!running && ['completed','cancelled','failed'].includes(scan.phase)) {
      clearTimeout(state.resultsTimer);
      state.resultsTimer = setTimeout(() => Promise.allSettled([refreshSummary(),refreshResults(),refreshHistory()]), 300);
    }
  } catch { $('#offline').hidden = false; }
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
  const query = new URLSearchParams({verdict:$('#verdict-filter').value, query:$('#search').value, limit:'150'});
  try {
    const rows = await api(`/api/results?${query}`);
    $('#results-empty').hidden = rows.length > 0;
    $('#results-body').innerHTML = rows.map(row => `<tr>
      <td>${badge(row.verdict, t(`${row.verdict}Badge`))}</td>
      <td class="path-cell"><code>${escapeHtml(row.path)}</code><details><summary>${t('details')}</summary>${escapeHtml(row.message)}</details></td>
      <td>${badge(row.authenticity)} </td>
      <td>${escapeHtml(row.format.toUpperCase())}<br><small class="muted">${formatBytes(row.size)}</small></td>
      <td>${escapeHtml(formatDate(row.checkedAt))}<br><small class="muted">${formatNumber(row.durationMs)} ms</small></td>
    </tr>`).join('');
  } catch {}
}

async function refreshHistory() {
  try {
    const rows = await api('/api/history');
    $('#history-list').innerHTML = rows.length ? rows.map(run => `<div class="history-row">
      <span>${badge(run.status)}</span>
      <span class="run-meta"><strong>${t(run.mode)}</strong><small>${formatDate(run.startedAt)}</small></span>
      <span class="number"><strong>${formatNumber(run.totalFiles)}</strong><br><small>${t('files')}</small></span>
      <span class="number"><strong>${formatNumber(run.skippedFiles)}</strong><br><small>${t('cached')}</small></span>
      <span class="number"><strong>${formatNumber(run.corruptFiles)}</strong><br><small>${t('defects')}</small></span>
    </div>`).join('') : `<div class="history-empty">${t('noResultsLead')}</div>`;
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
$('#verdict-filter').addEventListener('change', refreshResults);
$('#search').addEventListener('input', () => { clearTimeout(state.searchTimer); state.searchTimer=setTimeout(refreshResults,250); });
$$('.nav-link[href]').forEach(link => link.addEventListener('click', () => { $$('.nav-link').forEach(item => item.classList.remove('active')); link.classList.add('active'); }));

applyTheme(localStorage.getItem('audio-integrity-theme') || (matchMedia('(prefers-color-scheme:dark)').matches ? 'dark' : 'light'));
applyLanguage();
api('/api/session').then(session => session.authenticated ? showApp() : showLogin()).catch(showLogin);
