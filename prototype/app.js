const iconPaths = {
  pointer:'<path d="m5 3 13 8-6 1-3 6-4-15Z"/><path d="m13 13 4 5"/>',
  folder:'<path d="M3 7V5a1 1 0 0 1 1-1h5l2 3h9v12H3Z"/>',
  background:'<rect x="3" y="4" width="18" height="16" rx="2"/><path d="M3 8h18m-6 4 4 3-2 .5-1 2.5Z"/>',
  file:'<path d="M6 3h8l4 4v14H6Z"/><path d="M14 3v5h5M9 12h6m-6 4h6"/>',
  drive:'<rect x="3" y="6" width="18" height="12" rx="2"/><path d="M3 13h18m-4 2h1m-5 0h1"/>',
  grid:'<rect x="3" y="3" width="7" height="7" rx="1"/><rect x="14" y="3" width="7" height="7" rx="1"/><rect x="3" y="14" width="7" height="7" rx="1"/><rect x="14" y="14" width="7" height="7" rx="1"/>',
  history:'<path d="M3 10a9 9 0 1 1 2 8M3 4v6h6m3-4v6l4 2"/>',
  monitor:'<rect x="3" y="3" width="18" height="13" rx="2"/><path d="M12 16v5m-5 0h10"/>',
  shield:'<path d="m12 3 8 3v6c0 4-5 8-8 9-3-1-8-5-8-9V6Z"/><path d="m8 12 3 3 5-6"/>',
  help:'<circle cx="12" cy="12" r="9"/><path d="M9 9a3 3 0 1 1 4 3c-1 .4-1 1-1 2m0 3h.01"/>',
  info:'<circle cx="12" cy="12" r="9"/><path d="M12 11v6m0-10h.01"/>',
  plus:'<path d="M12 5v14M5 12h14"/>',
  search:'<circle cx="10.5" cy="10.5" r="6.5"/><path d="m16 16 5 5"/>',
  refresh:'<path d="M20 7V3m0 4h-4M4 17v4m0-4h4M4 9a8 8 0 0 1 14-5l2 3M4 17l2 3a8 8 0 0 0 14-5"/>',
  terminal:'<rect x="3" y="4" width="18" height="16" rx="2"/><path d="m7 9 3 3-3 3m6 0h4"/>',
  git:'<path d="m12 2 10 10-10 10L2 12Z"/><path d="m8 6 8 8M9 7v8"/><circle cx="9" cy="16" r="1"/><circle cx="16" cy="14" r="1"/>',
  code:'<path d="m8 6-6 6 6 6m8-12 6 6-6 6m-3-15-2 18"/>',
  spark:'<path d="m12 3 2.5 6.5L21 12l-6.5 2.5L12 21l-2.5-6.5L3 12l6.5-2.5Z"/>',
  lock:'<rect x="5" y="10" width="14" height="11" rx="2"/><path d="M8 10V7a4 4 0 0 1 8 0v3"/>',
  close:'<path d="m6 6 12 12M6 18 18 6"/>',
  chevron:'<path d="m9 5 7 7-7 7"/>',
  check:'<path d="m5 12 4 4L19 6"/>',
  settings:'<path d="M4 7h16M4 17h16"/><circle cx="9" cy="7" r="3"/><circle cx="15" cy="17" r="3"/>'
};
const icon = (name) => `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.65" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">${iconPaths[name] || iconPaths.terminal}</svg>`;
const escapeHtml = (value) => String(value).replace(/[&<>"']/g, (c) => ({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c]));
document.querySelectorAll('[data-icon]').forEach((el) => { el.innerHTML = icon(el.dataset.icon); });

const scopes = [
  {id:'all',name:'全部菜单',icon:'grid',description:'查看不同右键位置的菜单，整理常用命令。'},
  {id:'file',name:'文件',icon:'file',description:'右键点击文件时显示的菜单。'},
  {id:'folder',name:'文件夹',icon:'folder',description:'右键点击文件夹本身时显示的菜单。'},
  {id:'background',name:'文件夹空白处',icon:'background',description:'在文件夹内空白位置右键时显示的菜单。'},
  {id:'drive',name:'磁盘',icon:'drive',description:'右键点击磁盘时显示的菜单。'}
];
const entries = [
  {id:'terminal',scope:'background',name:'在终端中打开',source:'Windows',kind:'system',app:'terminal',enabled:true,command:'Windows Terminal',owner:'系统内置',reason:'系统内置项目。当前版本仅支持查看，避免影响基础操作。'},
  {id:'deveco',scope:'background',name:'Open Folder as DevEco Studio Project',source:'DevEco Studio',kind:'static',app:'deveco',enabled:true,command:'"C:\\Program Files\\Huawei\\DevEco Studio\\bin\\deveco64.exe" "%V"',owner:'所有用户'},
  {id:'dns',scope:'background',name:'Flush DNS',source:'自定义命令',kind:'static',app:'terminal',enabled:true,command:'cmd.exe /k ipconfig /flushdns',owner:'当前用户'},
  {id:'git-gui',scope:'background',name:'Open Git GUI here',source:'Git',kind:'static',app:'git',enabled:true,command:'"C:\\Program Files\\Git\\cmd\\git-gui.exe" "--working-dir" "%V"',owner:'所有用户'},
  {id:'git-bash',scope:'background',name:'Open Git Bash here',source:'Git',kind:'static',app:'git',enabled:true,command:'"C:\\Program Files\\Git\\git-bash.exe" "--cd=%V"',owner:'所有用户'},
  {id:'ip',scope:'background',name:'IP Config',source:'自定义命令',kind:'static',app:'terminal',enabled:true,command:'cmd.exe /k ipconfig',owner:'当前用户'},
  {id:'ping',scope:'background',name:'Ping Gateway',source:'自定义命令',kind:'static',app:'terminal',enabled:false,command:'cmd.exe /k ping 192.168.1.1',owner:'当前用户'},
  {id:'warp-tab',scope:'background',name:'Open Warp in new tab',source:'Warp',kind:'static',app:'warp',enabled:true,command:'"C:\\Users\\user\\AppData\\Local\\Programs\\Warp\\warp.exe" open --tab "%V"',owner:'当前用户'},
  {id:'warp-window',scope:'background',name:'Open Warp in new window',source:'Warp',kind:'static',app:'warp',enabled:true,command:'"C:\\Users\\user\\AppData\\Local\\Programs\\Warp\\warp.exe" open "%V"',owner:'当前用户'},
  {id:'workbuddy',scope:'background',name:'用 WorkBuddy 打开',source:'WorkBuddy',kind:'extension',app:'workbuddy',enabled:true,command:'WorkBuddy Shell Extension',owner:'所有用户',reason:'这个程序扩展可能生成多个菜单项。当前原型仅展示详情，具体支持范围将在扫描适配后确定。'},
  {id:'new',scope:'background',name:'新建',source:'Windows',kind:'system',app:'folder',enabled:true,command:'Windows Shell',owner:'系统内置',reason:'系统内置项目。当前版本仅支持查看，避免影响基础操作。'},
  {id:'file-code',scope:'file',name:'通过 Code 打开',source:'Visual Studio Code',kind:'static',app:'code',enabled:true,command:'"C:\\Users\\user\\AppData\\Local\\Programs\\Microsoft VS Code\\Code.exe" "%1"',owner:'当前用户'},
  {id:'file-zip',scope:'file',name:'7-Zip',source:'7-Zip',kind:'extension',app:'file',enabled:true,command:'7-Zip Shell Extension',owner:'所有用户',reason:'程序扩展可能生成多个子菜单。当前版本仅提供查看。'},
  {id:'file-open',scope:'file',name:'打开方式',source:'Windows',kind:'system',app:'file',enabled:true,command:'Windows Shell',owner:'系统内置',reason:'系统内置项目。当前版本仅提供查看。'},
  {id:'folder-bash',scope:'folder',name:'Open Git Bash here',source:'Git',kind:'static',app:'git',enabled:true,command:'"C:\\Program Files\\Git\\git-bash.exe" "--cd=%1"',owner:'所有用户'},
  {id:'folder-code',scope:'folder',name:'通过 Code 打开',source:'Visual Studio Code',kind:'static',app:'code',enabled:true,command:'"C:\\Users\\user\\AppData\\Local\\Programs\\Microsoft VS Code\\Code.exe" "%1"',owner:'当前用户'},
  {id:'folder-pin',scope:'folder',name:'固定到快速访问',source:'Windows',kind:'system',app:'folder',enabled:true,command:'Windows Shell',owner:'系统内置',reason:'系统内置项目。当前版本仅提供查看。'},
  {id:'drive-terminal',scope:'drive',name:'在终端中打开',source:'Windows',kind:'system',app:'terminal',enabled:true,command:'Windows Terminal',owner:'系统内置',reason:'系统内置项目。当前版本仅提供查看。'},
  {id:'drive-clean',scope:'drive',name:'磁盘清理',source:'自定义命令',kind:'static',app:'drive',enabled:true,command:'cleanmgr.exe /d "%1"',owner:'当前用户'}
];
const state = {scope:'background',filter:'all',query:'',selected:null,view:'menus',history:[]};
const scopeName = (id) => scopes.find((s) => s.id === id)?.name || id;
const manageable = (item) => item.kind === 'static';
const appIcon = (item) => `<span class="app-icon ${escapeHtml(item.app)}">${icon({deveco:'code',warp:'terminal',workbuddy:'spark',custom:'terminal'}[item.app] || item.app)}</span>`;
let toastTimer;
function toast(message){const el = document.getElementById('toast');el.textContent=message;el.hidden=false;clearTimeout(toastTimer);toastTimer=setTimeout(()=>{el.hidden=true},4500)}
function filteredEntries(){return entries.filter((e)=>(state.scope==='all'||e.scope===state.scope)&&(state.filter==='all'||state.filter==='enabled'&&e.enabled||state.filter==='disabled'&&!e.enabled||state.filter==='readonly'&&!manageable(e))&&`${e.name} ${e.source}`.toLowerCase().includes(state.query.toLowerCase()))}
function renderNav() {
  document.getElementById('scope-nav').innerHTML = scopes.map((s) => `<button class="nav-item ${state.view === 'menus' && state.scope === s.id ? 'active' : ''}" data-scope="${s.id}" ${state.view === 'menus' && state.scope === s.id ? 'aria-current="page"' : ''}>${icon(s.icon)}<span>${s.name}</span><span class="nav-count">${entries.filter((e) => s.id === 'all' || e.scope === s.id).length}</span></button>`).join('');
  document.getElementById('mobile-scope').innerHTML = scopes.map((s) => `<option value="${s.id}" ${state.scope === s.id ? 'selected' : ''}>${s.name}</option>`).join('');
  document.getElementById('history-nav').classList.toggle('active', state.view === 'history');
  document.getElementById('history-count').textContent = state.history.length;
}
function renderList() {
  const visible = filteredEntries();
  if (state.selected && !visible.some((e) => e.id === state.selected)) state.selected = null;
  document.getElementById('filter-all').textContent = entries.filter((e) => state.scope === 'all' || e.scope === state.scope).length;
  document.getElementById('list-count').textContent = `${visible.length} 个菜单项`;
  document.querySelectorAll('[data-filter]').forEach((button) => {
    button.classList.toggle('active', button.dataset.filter === state.filter);
    button.setAttribute('aria-pressed', String(button.dataset.filter === state.filter));
  });
  document.getElementById('menu-list').innerHTML = visible.length ? visible.map((e) => `
    <div class="menu-row ${state.selected === e.id ? 'selected' : ''} ${!e.enabled ? 'disabled' : ''}" data-id="${e.id}" tabindex="0" role="group" aria-label="${escapeHtml(e.name)}${state.selected === e.id ? '，当前选中' : ''}">
      <div class="menu-identity">${appIcon(e)}<div class="menu-text">
        <div class="menu-name" title="${escapeHtml(e.name)}">${escapeHtml(e.name)}</div>
        <div class="menu-subtitle">${state.scope === 'all' ? scopeName(e.scope) + ' · ' : ''}${e.kind === 'system' ? '系统内置' : e.kind === 'extension' ? '程序扩展 · 仅查看' : e.enabled ? '右键菜单中显示' : '已隐藏，可恢复'}</div>
      </div></div>
      <div class="menu-source">${escapeHtml(e.source)}</div>
      ${manageable(e) ? `<button class="switch" role="switch" aria-checked="${e.enabled}" aria-label="${e.enabled ? '停用' : '启用'} ${escapeHtml(e.name)}" data-toggle="${e.id}"></button>` : `<span class="readonly-badge">${icon('lock')}仅查看</span>`}
      <button class="icon-button row-detail" aria-label="查看 ${escapeHtml(e.name)} 的详情" data-detail="${e.id}" title="查看详情">${icon('chevron')}</button>
    </div>`).join('') : `<div class="empty-state">${icon('search')}<strong>没有找到匹配的菜单</strong>试试其他关键词或切换状态筛选。</div>`;
  renderDetail();
}
function renderDetail(){const el=document.getElementById('inspector');const e=entries.find((x)=>x.id===state.selected);if(state.view==='history'){el.innerHTML=`<div class="inspector-label">恢复说明 ${icon('shield')}</div><h2 style="font-size:16px;font-weight:600">每次修改，都留有退路</h2><p class="history-intro">停用前保存原始状态。你可以在操作历史里找到对应记录，并恢复到修改前。</p><p class="history-intro">正式版恢复时会检查菜单是否已被其他软件改动，出现冲突时提示处理。</p><div class="confirmation-note">${icon('info')}<span>当前为界面原型，历史记录只在本次页面打开期间保留。</span></div>`;return}if(!e){el.innerHTML=`<div class="empty-state">${icon('pointer')}选择一个菜单项查看详情</div>`;return}const previewItems=entries.filter((x)=>x.scope===e.scope&&(manageable(x)||x.id===e.id));let near=previewItems.findIndex((x)=>x.id===e.id);const preview=previewItems.slice(Math.max(0,near-1),Math.max(0,near-1)+3);const regRoot=e.owner==='所有用户'?'HKEY_LOCAL_MACHINE':'HKEY_CURRENT_USER';const regScopes={background:'Directory\\Background',folder:'Directory',file:'*',drive:'Drive'};el.innerHTML=`<div class="inspector-label">菜单详情 ${icon('settings')}</div><div class="detail-title">${appIcon(e)}<div><h2>${escapeHtml(e.name)}</h2><p>${escapeHtml(e.source)}</p></div></div><div class="detail-status ${!e.enabled?'off':''}"><span class="status-dot"></span>${e.enabled?'已启用':'已停用'}</div><dl class="detail-facts"><div><dt>显示位置</dt><dd>${scopeName(e.scope)}</dd></div><div><dt>适用范围</dt><dd>${e.owner}</dd></div><div><dt>菜单类型</dt><dd>${e.kind==='static'?'普通命令':e.kind==='extension'?'程序扩展':'系统内置'}</dd></div></dl><div class="path-label">${e.kind==='static'?'执行命令':'关联组件'}</div><div class="command-path">${escapeHtml(e.command)}</div><details class="technical"><summary>查看注册位置（示例）</summary><p>${escapeHtml(regRoot+'\\Software\\Classes\\'+regScopes[e.scope]+'\\'+(e.kind==='extension'?'shellex\\ContextMenuHandlers':'shell')+'\\'+e.id)}</p></details><div class="preview-heading"><strong>右键菜单预览</strong><span>示意 · 非系统截图</span></div><div class="context-preview"><div class="context-menu"><div class="context-item">${icon('grid')}查看<span style="margin-left:auto">›</span></div><div class="context-item">${icon('refresh')}刷新</div><div class="context-separator"></div>${preview.map((x)=>`<div class="context-item ${x.id===e.id?'highlight':''} ${!x.enabled?'gone':''}" title="${escapeHtml(x.name)}">${icon(x.app==='git'?'git':'terminal')}<span style="overflow:hidden;text-overflow:ellipsis">${escapeHtml(x.name)}</span></div>`).join('')}<div class="context-separator"></div><div class="context-item">${icon('folder')}新建<span style="margin-left:auto">›</span></div></div></div><p class="preview-hint">${e.enabled?'蓝色高亮为当前选中的菜单项。':'划线项目表示已停用，实际菜单中将不再显示。'}</p>${manageable(e)?`<button class="button ${e.enabled?'secondary':'primary'} detail-action" data-toggle="${e.id}">${icon(e.enabled?'lock':'refresh')}${e.enabled?'停用这个菜单':'恢复这个菜单'}</button><p class="detail-note">${e.owner==='所有用户'?'修改此项目需要管理员权限。':''}停用前会保存原始状态，可在操作历史中恢复。</p>`:`<p class="detail-note">${escapeHtml(e.reason)}</p>`}`}
function render(){renderNav();const history=state.view==='history';document.getElementById('menu-view').hidden=history;document.getElementById('history-view').hidden=!history;document.getElementById('add-button').hidden=history;document.getElementById('page-title').textContent=history?'操作历史':scopeName(state.scope);document.getElementById('eyebrow').textContent=history?'本地记录 / 操作历史':`菜单管理 / ${scopeName(state.scope)}`;document.getElementById('page-description').textContent=history?'查看本次原型体验中的修改，随时恢复。':scopes.find((s)=>s.id===state.scope).description;if(history)renderHistory();else renderList();renderDetail()}
function updateDialogAccess() {
  const panel = document.getElementById('detail-panel');
  const modalOpen = Boolean(document.querySelector('.modal'));
  document.querySelectorAll('.main,.sidebar,.titlebar,.statusbar').forEach((el) => { el.inert = modalOpen || !panel.hidden; });
  panel.inert = modalOpen;
}
function openModal(body, wide = false) {
  const root = document.getElementById('modal-root');
  root.previousFocus = document.activeElement;
  root.innerHTML = `<div class="modal-backdrop"><section class="modal ${wide ? 'wide' : ''}" role="dialog" aria-modal="true" aria-labelledby="modal-title">${body}</section></div>`;
  updateDialogAccess();
  root.querySelector('input,select,button')?.focus();
  root.querySelector('.modal-backdrop').addEventListener('click', (event) => { if (event.target === event.currentTarget) closeModal(); });
  root.querySelectorAll('[data-close]').forEach((button) => button.addEventListener('click', closeModal));
}
function closeModal() {
  const root = document.getElementById('modal-root');
  root.innerHTML = '';
  updateDialogAccess();
  if (root.previousFocus?.isConnected) root.previousFocus.focus();
  else if (!document.getElementById('detail-panel').hidden) document.getElementById('detail-close').focus();
  else document.querySelector(`[data-id="${state.selected}"]`)?.focus();
}
const modalHeading=(title)=>`<div class="modal-top"><h2 id="modal-title">${title}</h2><button class="icon-button" aria-label="关闭对话框" data-close>${icon('close')}</button></div>`;
function changeEntry(id){const item=entries.find((e)=>e.id===id);if(!item||!manageable(item))return;state.selected=id;if(item.enabled){openModal(`${modalHeading('停用这个菜单？')}<p class="modal-description">这个项目将从右键菜单中隐藏，之后可以随时恢复。</p><div class="confirmation-target">${escapeHtml(item.name)}<div class="confirmation-scope">${scopeName(item.scope)} · ${item.owner}</div></div><div class="confirmation-note">${icon('shield')}<span>会先保存原始状态，再执行停用。${item.owner==='所有用户'?'正式版操作时将请求管理员权限。':''}<br>当前仅模拟操作，不修改系统。</span></div><div class="modal-actions"><button class="button secondary" data-close>取消</button><button class="button primary" id="confirm-disable">停用菜单</button></div>`);document.getElementById('confirm-disable').addEventListener('click',()=>{applyChange(item,false);closeModal()})}else applyChange(item,true)}
function applyChange(item,enabled){state.history.unshift({id:crypto.randomUUID(),entryId:item.id,name:item.name,scope:item.scope,type:enabled?'enable':'disable',before:item.enabled,after:enabled,time:new Date(),restored:false});item.enabled=enabled;render();toast(`${enabled?'已恢复':'已停用'}「${item.name}」· 模拟操作`)}
function renderHistory(){document.getElementById('history-view').innerHTML=state.history.length?state.history.map((h)=>`<div class="history-row">${icon(h.type==='create'?'plus':h.type==='disable'?'lock':'refresh')}<div><strong>${h.type==='create'?'添加':h.type==='disable'?'停用':'恢复'} · ${escapeHtml(h.name)}</strong><p>${scopeName(h.scope)} · ${h.time.toLocaleTimeString('zh-CN',{hour:'2-digit',minute:'2-digit'})} · ${h.restored?'已撤销':'已保存修改前的状态'}</p></div>${h.restored?'<span class="history-restored">已撤销</span>':`<button class="button secondary" data-restore="${h.id}">${h.type==='create'?'撤销添加':'恢复原状态'}</button>`}</div>`).join(''):`<div class="empty-state">${icon('history')}<strong>还没有修改记录</strong>停用或添加一个菜单，记录会显示在这里。</div>`}
function restoreHistory(id){const h=state.history.find((x)=>x.id===id);if(!h||h.restored)return;const item=entries.find((e)=>e.id===h.entryId);if(!item||item.enabled!==h.after){toast('菜单已发生其他变化，不能直接覆盖。请先撤销后续操作。');return}if(h.type==='create'){if(state.history.some((x)=>x.entryId===h.entryId&&x.time>h.time&&!x.restored)){toast('请先撤销这个菜单的后续操作。');return}entries.splice(entries.indexOf(item),1)}else item.enabled=h.before;h.restored=true;render();toast('已恢复修改前的状态 · 模拟操作')}
function openAdd(){openModal(`${modalHeading('添加自定义菜单')}<p class="modal-description">把常用程序放进右键菜单，直接打开当前文件或文件夹。</p><form id="add-form"><label class="field"><span>菜单名称</span><input name="name" placeholder="例如：用 VS Code 打开" maxlength="80" required /></label><label class="field"><span>显示位置</span><select name="scope">${scopes.filter((s)=>s.id!=='all').map((s)=>`<option value="${s.id}" ${s.id===state.scope?'selected':''}>${s.name}</option>`).join('')}</select></label><label class="field"><span>程序或脚本路径</span><input name="program" placeholder="C:\\Program Files\\App\\app.exe" required /><small>原型中可直接填写示例路径，无需文件实际存在。</small></label><label class="field"><span>启动参数 <span class="field-optional">（可选）</span></span><input name="args" placeholder='"{path}"' /><small>使用 {path} 代表右键选中的路径；建议写成 "{path}" 以支持含空格的路径。</small></label><div class="confirmation-note" style="margin:19px 0 0">${icon('info')}<span>添加到当前用户的菜单。此处仅模拟添加，不写入注册表。</span></div><p id="form-error" class="form-error" role="alert" hidden></p><div class="modal-actions"><button type="button" class="button secondary" data-close>取消</button><button type="submit" class="button primary">添加菜单</button></div></form>`,true);document.getElementById('add-form').addEventListener('submit',(event)=>{event.preventDefault();const data=new FormData(event.currentTarget);const name=data.get('name').trim();const program=data.get('program').trim();if(!name||!program){const error=document.getElementById('form-error');error.textContent='请填写菜单名称和程序路径。';error.hidden=false;return}const item={id:'custom-'+crypto.randomUUID(),scope:data.get('scope'),name,source:'自定义命令',kind:'static',app:'custom',enabled:true,command:`"${program}" ${data.get('args').trim()||'"{path}"'}`,owner:'当前用户'};entries.push(item);state.history.unshift({id:crypto.randomUUID(),entryId:item.id,name,scope:item.scope,type:'create',before:null,after:true,time:new Date(),restored:false});state.scope=item.scope;state.filter='all';state.query='';state.selected=item.id;state.view='menus';document.getElementById('search-input').value='';closeModal();render();toast(`已添加「${name}」· 模拟操作`)})}
function changeScope(id) {
  state.scope = id; state.view = 'menus'; state.filter = 'all'; state.query = ''; state.selected = null;
  document.getElementById('search-input').value = '';
  render();
}
document.getElementById('scope-nav').addEventListener('click', (event) => { const button = event.target.closest('[data-scope]'); if (button) changeScope(button.dataset.scope); });
document.getElementById('mobile-scope').addEventListener('change', (event) => changeScope(event.target.value));
document.getElementById('history-nav').addEventListener('click',()=>{state.view='history';render()});
document.getElementById('filters').addEventListener('click',(event)=>{const b=event.target.closest('[data-filter]');if(!b)return;state.filter=b.dataset.filter;renderList()});
document.getElementById('search-input').addEventListener('input',(event)=>{state.query=event.target.value;renderList()});
function selectEntry(id) {
  state.selected = id;
  renderList();
  document.getElementById('detail-panel').hidden = false;
  document.getElementById('detail-backdrop').hidden = false;
  updateDialogAccess();
  document.getElementById('detail-close').focus();
}
function closeDetail() {
  document.getElementById('detail-panel').hidden = true;
  document.getElementById('detail-backdrop').hidden = true;
  updateDialogAccess();
  document.querySelector(`[data-id="${state.selected}"]`)?.focus();
}
document.getElementById('detail-close').addEventListener('click',closeDetail);
document.getElementById('detail-backdrop').addEventListener('click',closeDetail);
document.getElementById('menu-list').addEventListener('click',(event)=>{const toggle=event.target.closest('[data-toggle]');if(toggle){changeEntry(toggle.dataset.toggle);return}const row=event.target.closest('[data-id]');if(row)selectEntry(row.dataset.id)});
document.getElementById('menu-list').addEventListener('keydown',(event)=>{if(event.target.matches('.menu-row')&&(event.key==='Enter'||event.key===' ')){event.preventDefault();selectEntry(event.target.dataset.id)}});
document.getElementById('inspector').addEventListener('click',(event)=>{const b=event.target.closest('[data-toggle]');if(b)changeEntry(b.dataset.toggle)});
document.getElementById('history-view').addEventListener('click',(event)=>{const b=event.target.closest('[data-restore]');if(b)restoreHistory(b.dataset.restore)});
document.getElementById('add-button').addEventListener('click',openAdd);
document.getElementById('refresh-button').addEventListener('click',()=>{const b=document.getElementById('refresh-button');b.disabled=true;b.classList.add('spin');document.getElementById('scan-status').textContent='正在模拟扫描…';setTimeout(()=>{b.disabled=false;b.classList.remove('spin');document.getElementById('scan-status').textContent='示例扫描已完成';toast('示例菜单已刷新。原型不会读取真实系统。');render()},650)});
document.getElementById('help-button').addEventListener('click',()=>openModal(`${modalHeading('关于这个原型')}<p class="modal-description">用于确认布局与操作流程。所有菜单、程序路径和状态都是示例，不代表真实扫描结果。</p><ul class="help-list"><li>切换左侧分类，或搜索菜单与来源。</li><li>选择一项，在右侧查看详情和菜单示意。</li><li>点击开关，体验停用、恢复和操作历史。</li><li>点击“添加菜单”，体验自定义菜单流程。</li></ul><div class="modal-actions"><button class="button primary" data-close>知道了</button></div>`));
document.addEventListener('keydown', (event) => {
  const modal = document.querySelector('.modal');
  const panel = document.getElementById('detail-panel');
  if (event.key === 'Escape') {
    if (modal) closeModal();
    else if (!panel.hidden) closeDetail();
    return;
  }
  const activeDialog = modal || (!panel.hidden ? panel : null);
  if (activeDialog && event.key === 'Tab') {
    const items = [...activeDialog.querySelectorAll('button:not(:disabled),input,select,summary,[tabindex="0"]')].filter((el) => el.offsetParent !== null);
    if (!items.length) return;
    const first = items[0], last = items.at(-1);
    if (event.shiftKey && document.activeElement === first) { event.preventDefault(); last.focus(); }
    else if (!event.shiftKey && document.activeElement === last) { event.preventDefault(); first.focus(); }
  }
  if (event.key === '/' && !activeDialog && !['INPUT', 'TEXTAREA', 'SELECT'].includes(document.activeElement.tagName) && state.view === 'menus') {
    event.preventDefault(); document.getElementById('search-input').focus();
  }
});
render();
