import { useCallback, useEffect, useRef, useState, type FormEvent } from 'react';
import { open } from '@tauri-apps/plugin-dialog';
import {
  ChevronRight,
  CircleHelp,
  CircleAlert,
  History,
  Info,
  LoaderCircle,
  LockKeyhole,
  Monitor,
  MousePointer2,
  Plus,
  RefreshCw,
  Search,
  ShieldCheck,
  X,
} from 'lucide-react';
import { api, native } from './api';
import { AppIcon, Dialog, ScopeIcon } from './components';
import type { AppData, HistoryItem, MenuEntry, NewMenu, Scope } from './types';

const scopes: { id: Scope; name: string; description: string }[] = [
  { id: 'all', name: '全部菜单', description: '查看各个右键位置的注册项，整理常用命令。' },
  { id: 'file', name: '文件', description: '右键点击任意文件时适用的菜单注册项。' },
  { id: 'folder', name: '文件夹', description: '右键点击文件夹本身时适用的菜单注册项。' },
  {
    id: 'background',
    name: '文件夹空白处',
    description: '在文件夹内空白位置右键时适用的菜单注册项。',
  },
  { id: 'drive', name: '磁盘', description: '右键点击磁盘时适用的菜单注册项。' },
  { id: 'filesystem', name: '文件与文件夹', description: '同时适用于文件和文件夹的菜单注册项。' },
];
const scopeName = (id: Scope) => scopes.find((s) => s.id === id)!.name;
const operationName = (operation: string) =>
  ({ create: '添加', disable: '停用', enable: '启用' })[operation] || operation;
const statusName = (status: string) =>
  ({
    applied: '已完成',
    prepared: '结果待检查',
    failed: '操作失败',
    restored: '已恢复',
    restore_pending: '恢复待检查',
  })[status] || status;
type Notice = { kind: 'error' | 'success'; text: string };
type Confirmation = { type: 'toggle'; entry: MenuEntry } | { type: 'restore'; item: HistoryItem };

export default function App() {
  const [data, setData] = useState<AppData | null>(null);
  const [scope, setScope] = useState<Scope>('background');
  const [filter, setFilter] = useState('all');
  const [query, setQuery] = useState('');
  const [view, setView] = useState<'menus' | 'history'>('menus');
  const [selected, setSelected] = useState<string | null>(null);
  const [confirmation, setConfirmation] = useState<Confirmation | null>(null);
  const [adding, setAdding] = useState(false);
  const [help, setHelp] = useState(false);
  const [loading, setLoading] = useState(true);
  const [busy, setBusy] = useState(false);
  const [notice, setNotice] = useState<Notice | null>(null);
  const search = useRef<HTMLInputElement>(null);
  const refresh = useCallback(async () => {
    if (!native) {
      setLoading(false);
      return;
    }
    setLoading(true);
    try {
      setData(await api.load());
    } finally {
      setLoading(false);
    }
  }, []);
  useEffect(() => {
    refresh().catch((error) => setNotice({ kind: 'error', text: String(error) }));
  }, [refresh]);
  useEffect(() => {
    const keydown = (event: KeyboardEvent) => {
      if (
        event.key === '/' &&
        !document.querySelector('[data-modal]') &&
        !['INPUT', 'TEXTAREA', 'SELECT'].includes((event.target as HTMLElement).tagName)
      ) {
        event.preventDefault();
        search.current?.focus();
      }
    };
    document.addEventListener('keydown', keydown);
    return () => document.removeEventListener('keydown', keydown);
  }, []);
  const entries = data?.scan.entries || [];
  const inScope = entries.filter((e) => scope === 'all' || e.scope === scope);
  const visible = inScope.filter(
    (e) =>
      `${e.name} ${e.source} ${e.command}`.toLowerCase().includes(query.toLowerCase()) &&
      (filter === 'all' ||
        (filter === 'enabled' && e.enabled) ||
        (filter === 'disabled' && !e.enabled) ||
        (filter === 'readonly' && !e.manageable)),
  );
  const entry = entries.find((e) => e.id === selected);
  const navigate = (next: Scope) => {
    setScope(next);
    setView('menus');
    setFilter('all');
    setQuery('');
    setSelected(null);
  };
  async function operate(action: () => Promise<void>, message: string) {
    setBusy(true);
    setNotice(null);
    try {
      await action();
      setNotice({ kind: 'success', text: message });
      setConfirmation(null);
      setAdding(false);
    } catch (error) {
      setNotice({ kind: 'error', text: String(error) });
      setConfirmation(null);
    } finally {
      try {
        await refresh();
      } catch (error) {
        setNotice({ kind: 'error', text: `无法刷新操作结果：${error}。请重新扫描或重启应用。` });
      }
      setBusy(false);
    }
  }
  const beginToggle = (e: MenuEntry) => setConfirmation({ type: 'toggle', entry: e });
  const confirmAdmin =
    confirmation?.type === 'toggle'
      ? confirmation.entry.requiresAdmin
      : confirmation?.item.requiresAdmin;
  const confirmLabel =
    confirmation?.type === 'toggle'
      ? `${confirmation.entry.enabled ? '停用' : '启用'}菜单`
      : confirmation?.item.operation === 'create'
        ? '撤销添加'
        : '恢复原状态';
  const title = view === 'history' ? '操作历史' : scopeName(scope);

  return (
    <>
      <div className="app" id="app-content">
        <header className="titlebar">
          <div className="brand">
            <span className="brand-icon">
              <MousePointer2 />
            </span>
            <strong>右键管理</strong>
            <span className="version">0.1.0</span>
          </div>
          <div className="prototype-label">
            <span className="status-dot" />
            本地管理 · Windows
          </div>
          <button className="icon-button" aria-label="使用说明" onClick={() => setHelp(true)}>
            <CircleHelp />
          </button>
        </header>
        <div className="workspace">
          <aside className="sidebar">
            <div className="sidebar-label">菜单位置</div>
            <nav id="scope-nav" aria-label="菜单位置">
              {scopes.map((s) => (
                <button
                  key={s.id}
                  className={`nav-item ${view === 'menus' && scope === s.id ? 'active' : ''}`}
                  aria-current={view === 'menus' && scope === s.id ? 'page' : undefined}
                  onClick={() => navigate(s.id)}
                >
                  <ScopeIcon scope={s.id} />
                  <span>{s.name}</span>
                  <span className="nav-count">
                    {entries.filter((e) => s.id === 'all' || e.scope === s.id).length}
                  </span>
                </button>
              ))}
            </nav>
            <label className="mobile-scope">
              <span>菜单位置</span>
              <select
                aria-label="选择菜单位置"
                value={scope}
                onChange={(e) => navigate(e.target.value as Scope)}
              >
                {scopes.map((s) => (
                  <option key={s.id} value={s.id}>
                    {s.name}
                  </option>
                ))}
              </select>
            </label>
            <div className="sidebar-divider" />
            <button
              className={`nav-item ${view === 'history' ? 'active' : ''}`}
              id="history-nav"
              onClick={() => {
                setView('history');
                setSelected(null);
              }}
            >
              <History />
              <span>操作历史</span>
              <span className="nav-count">{data?.history.length || 0}</span>
            </button>
            <div className="sidebar-bottom">
              <div className="computer">
                <Monitor />
                <div>
                  <strong>这台电脑</strong>
                  <small>传统右键菜单 · 64 位视图</small>
                </div>
              </div>
              <div className="local-note">
                <ShieldCheck />
                所有记录保存在本地
              </div>
            </div>
          </aside>
          <main className="main">
            <div className="page-heading">
              <div>
                <h1>{title}</h1>
                <p>
                  {view === 'history'
                    ? '查看修改记录，恢复到操作前的状态。'
                    : scopes.find((s) => s.id === scope)!.description}
                </p>
              </div>
              {view === 'menus' && (
                <button
                  className="button primary"
                  disabled={!native || busy}
                  onClick={() => setAdding(true)}
                >
                  <Plus />
                  添加菜单
                </button>
              )}
            </div>
            {!native && (
              <div className="notice">
                <Info />
                <span>
                  请通过桌面应用使用真实扫描与菜单管理。浏览器仅能查看应用界面，原型可在 4173
                  端口体验。
                </span>
              </div>
            )}
            {notice && (
              <div
                className={`notice ${notice.kind}`}
                role={notice.kind === 'error' ? 'alert' : 'status'}
              >
                {notice.kind === 'error' ? <CircleAlert /> : <ShieldCheck />}
                <span>{notice.text}</span>
                <button
                  className="icon-button"
                  aria-label="关闭提示"
                  onClick={() => setNotice(null)}
                >
                  <X />
                </button>
              </div>
            )}
            {data && data.scan.warnings.length > 0 && (
              <details className="scan-warning">
                <summary>有 {data.scan.warnings.length} 项读取提示，点击查看</summary>
                <ul>
                  {data.scan.warnings.map((warning, i) => (
                    <li key={i}>{warning}</li>
                  ))}
                </ul>
              </details>
            )}
            {view === 'menus' ? (
              <>
                <div className="toolbar">
                  <label className="search">
                    <Search />
                    <input
                      ref={search}
                      type="search"
                      value={query}
                      onChange={(e) => setQuery(e.target.value)}
                      placeholder="搜索菜单名称、来源或命令"
                      aria-label="搜索菜单"
                    />
                    <kbd>/</kbd>
                  </label>
                  <button
                    className="button secondary"
                    disabled={!native || loading || busy}
                    onClick={() => {
                      setNotice(null);
                      refresh().catch((error) => setNotice({ kind: 'error', text: String(error) }));
                    }}
                  >
                    <RefreshCw className={loading ? 'spinning' : ''} />
                    {loading ? '正在扫描' : '重新扫描'}
                  </button>
                </div>
                <div className="list-topline">
                  <div className="filters" aria-label="状态筛选">
                    {[
                      ['all', '全部'],
                      ['enabled', '已启用'],
                      ['disabled', '已停用'],
                      ['readonly', '仅查看'],
                    ].map(([id, label]) => (
                      <button
                        key={id}
                        className={filter === id ? 'active' : ''}
                        aria-pressed={filter === id}
                        onClick={() => setFilter(id)}
                      >
                        {label}
                        {id === 'all' && <span>{inScope.length}</span>}
                      </button>
                    ))}
                  </div>
                  <span className="list-count">{visible.length} 个注册项</span>
                </div>
                <div className="column-head">
                  <span>菜单名称</span>
                  <span>来源</span>
                  <span>启用状态</span>
                  <span />
                </div>
                <div className="menu-list" aria-label="菜单列表">
                  {loading && !data ? (
                    <div className="empty-state">
                      <LoaderCircle className="spinning" />
                      <strong>正在读取本机菜单</strong>扫描不会修改系统。
                    </div>
                  ) : visible.length ? (
                    visible.map((e) => (
                      <div
                        key={e.id}
                        className={`menu-row ${selected === e.id ? 'selected' : ''} ${!e.enabled ? 'disabled' : ''}`}
                        role="group"
                        aria-label={e.name}
                        tabIndex={0}
                        onClick={() => setSelected(e.id)}
                        onKeyDown={(event) => {
                          if (
                            event.target === event.currentTarget &&
                            ['Enter', ' '].includes(event.key)
                          ) {
                            event.preventDefault();
                            setSelected(e.id);
                          }
                        }}
                      >
                        <div className="menu-identity">
                          <AppIcon entry={e} />
                          <div className="menu-text">
                            <div className="menu-name">{e.name}</div>
                            <div className="menu-subtitle">
                              {scope === 'all' ? `${scopeName(e.scope)} · ` : ''}
                              {e.kind === 'system'
                                ? '系统基础菜单'
                                : e.kind === 'extension'
                                  ? '程序扩展 · 仅查看'
                                  : !e.manageable
                                    ? '暂不支持修改'
                                    : e.enabled
                                      ? '注册已启用'
                                      : '已停用，可恢复'}
                              {e.extended ? ' · Shift 右键' : ''}
                            </div>
                          </div>
                        </div>
                        <div className="menu-source">{e.source}</div>
                        {e.manageable ? (
                          <button
                            className="switch"
                            role="switch"
                            aria-checked={e.enabled}
                            aria-label={`${e.enabled ? '停用' : '启用'} ${e.name}`}
                            disabled={busy}
                            onClick={(event) => {
                              event.stopPropagation();
                              beginToggle(e);
                            }}
                          />
                        ) : (
                          <span className="readonly-badge">
                            <LockKeyhole />
                            仅查看
                          </span>
                        )}
                        <button
                          className="icon-button row-detail"
                          aria-label={`查看 ${e.name} 的详情`}
                          onClick={(event) => {
                            event.stopPropagation();
                            setSelected(e.id);
                          }}
                        >
                          <ChevronRight />
                        </button>
                      </div>
                    ))
                  ) : (
                    <div className="empty-state">
                      <Search />
                      <strong>
                        {query || filter !== 'all'
                          ? '没有找到匹配的菜单'
                          : '这个位置没有扫描到注册项'}
                      </strong>
                      {query || filter !== 'all'
                        ? '试试其他关键词或切换状态筛选。'
                        : native
                          ? '第一版扫描常见的传统菜单位置，部分菜单可能由其他机制生成。'
                          : '运行桌面应用后，这里会显示真实数据。'}
                    </div>
                  )}
                </div>
                <div className="list-footer">
                  <Info />
                  <span>
                    这里显示注册状态。菜单是否实际出现，还取决于所选对象、程序规则和资源管理器缓存。
                  </span>
                </div>
              </>
            ) : (
              <>
                {data?.history.length ? (
                  data.history.map((item) => (
                    <div className="history-row" key={item.id}>
                      <History />
                      <div>
                        <strong>
                          {operationName(item.operation)} · {item.label}
                        </strong>
                        <p>
                          {scopeName(item.scope)} ·{' '}
                          {new Date(item.timestamp).toLocaleString('zh-CN')} ·{' '}
                          {statusName(item.status)}
                        </p>
                        {item.error && <p className="history-error">{item.error}</p>}
                      </div>
                      {item.canRestore ? (
                        <button
                          className="button secondary"
                          disabled={busy}
                          onClick={() => setConfirmation({ type: 'restore', item })}
                        >
                          {item.operation === 'create' ? '撤销添加' : '恢复原状态'}
                        </button>
                      ) : (
                        <span className="history-restored">已恢复</span>
                      )}
                    </div>
                  ))
                ) : (
                  <div className="empty-state">
                    <History />
                    <strong>还没有修改记录</strong>停用、启用或添加菜单后，这里会保存恢复记录。
                  </div>
                )}
                <div className="list-footer">
                  <ShieldCheck />
                  <span>
                    恢复前会检查菜单是否发生变化。如有冲突，先恢复后续操作。记录位置：
                    <code>{data?.storagePath || '桌面应用的数据目录'}</code>
                  </span>
                </div>
              </>
            )}
          </main>
        </div>
        <footer className="statusbar">
          <span>
            <span className="status-dot" />
            {busy
              ? '正在处理，请留意管理员授权窗口…'
              : loading
                ? '正在扫描菜单…'
                : data
                  ? `已读取 ${entries.length} 个菜单注册项`
                  : '尚未读取本机数据'}
          </span>
          <span>
            {data
              ? `上次扫描 ${new Date(data.scan.scannedAt).toLocaleTimeString('zh-CN')}`
              : '所有修改均保存恢复记录'}
          </span>
        </footer>
      </div>
      {entry && (
        <Dialog title="菜单详情" drawer busy={busy} onClose={() => setSelected(null)}>
          <div className="inspector">
            <div className="inspector-label">菜单详情</div>
            {notice?.kind === 'error' && (
              <div className="notice error" role="alert">
                <CircleAlert />
                <span>{notice.text}</span>
              </div>
            )}
            <div className="detail-title">
              <AppIcon entry={entry} />
              <div>
                <h2>{entry.name}</h2>
                <p>{entry.source}</p>
              </div>
            </div>
            <div className={`detail-status ${entry.enabled ? '' : 'off'}`}>
              <span className="status-dot" />
              {entry.enabled ? '注册已启用' : '注册已停用'}
            </div>
            <dl className="detail-facts">
              <div>
                <dt>显示位置</dt>
                <dd>{scopeName(entry.scope)}</dd>
              </div>
              <div>
                <dt>适用范围</dt>
                <dd>{entry.owner}</dd>
              </div>
              <div>
                <dt>菜单类型</dt>
                <dd>
                  {entry.kind === 'extension'
                    ? '程序扩展'
                    : entry.kind === 'system'
                      ? '系统基础菜单'
                      : '普通命令'}
                </dd>
              </div>
              {entry.extended && (
                <div>
                  <dt>触发条件</dt>
                  <dd>按住 Shift 右键</dd>
                </div>
              )}
            </dl>
            {entry.manageable ? (
              <>
                <button
                  className={`button ${entry.enabled ? 'secondary' : 'primary'} detail-action`}
                  disabled={busy}
                  onClick={() => beginToggle(entry)}
                >
                  {entry.enabled ? <LockKeyhole /> : <RefreshCw />}
                  {entry.enabled ? '停用这个菜单' : '启用这个菜单'}
                </button>
                <p className="detail-note">
                  {entry.requiresAdmin ? '修改此项目需要管理员授权。' : ''}
                  操作前保存原始状态，可在历史记录中恢复。
                </p>
              </>
            ) : (
              <p className="detail-note">{entry.reason}</p>
            )}
            <div className="detail-section">
              <div className="path-label">
                {entry.kind === 'extension' ? '关联组件标识' : '执行命令'}
              </div>
              <div className="command-path">{entry.command || '未读取到普通执行命令'}</div>
              <details className="technical">
                <summary>查看注册位置</summary>
                <p>{entry.registryPath}</p>
              </details>
            </div>
            <div className="preview-heading">
              <strong>菜单显示示意</strong>
              <span>并非系统截图</span>
            </div>
            <div className="context-preview">
              <div className="context-menu">
                <div className="context-item">其他菜单项</div>
                <div className="context-separator" />
                <div className={`context-item ${entry.enabled ? 'highlight' : 'gone'}`}>
                  <ScopeIcon scope={entry.scope} />
                  <span>{entry.name}</span>
                </div>
                <div className="context-separator" />
                <div className="context-item">其他菜单项</div>
              </div>
            </div>
            <p className="preview-hint">
              {entry.enabled
                ? '注册已启用，实际是否显示请在资源管理器中确认。'
                : '普通命令被停用后，将从相应菜单中隐藏。'}
              {entry.kind === 'extension' ? ' 一个程序扩展可能生成多个项目。' : ''}
            </p>
          </div>
        </Dialog>
      )}
      {confirmation && (
        <Dialog
          title={
            confirmation.type === 'toggle'
              ? `${confirmation.entry.enabled ? '停用' : '启用'}这个菜单？`
              : '恢复修改前的状态？'
          }
          busy={busy}
          onClose={() => setConfirmation(null)}
        >
          <p className="modal-description">
            {confirmation.type === 'toggle'
              ? confirmation.entry.enabled
                ? '这个普通命令将从右键菜单中隐藏，之后可以恢复。'
                : '这个普通命令将重新启用，之后可以恢复到本次修改前。'
              : '会先检查当前菜单是否与记录一致，再恢复原始状态。'}
          </p>
          <div className="confirmation-target">
            {confirmation.type === 'toggle' ? confirmation.entry.name : confirmation.item.label}
            <div className="confirmation-scope">
              {scopeName(
                confirmation.type === 'toggle' ? confirmation.entry.scope : confirmation.item.scope,
              )}
            </div>
          </div>
          <div className="confirmation-note">
            <ShieldCheck />
            <span>
              {confirmAdmin
                ? '接下来 Windows 将请求管理员授权。只有受限的辅助进程以管理员权限运行。'
                : '操作仅作用于当前用户。'}
              {confirmation.type === 'toggle'
                ? '修改前会先保存恢复记录。'
                : '发生冲突时不会覆盖现有配置。'}
            </span>
          </div>
          <div className="modal-actions">
            <button
              className="button secondary"
              disabled={busy}
              onClick={() => setConfirmation(null)}
            >
              取消
            </button>
            <button
              className="button primary"
              disabled={busy}
              onClick={() =>
                confirmation.type === 'toggle'
                  ? operate(
                      () => api.toggle(confirmation.entry.id, !confirmation.entry.enabled),
                      '菜单状态已修改，原始状态已保存在操作历史中。',
                    )
                  : operate(() => api.restore(confirmation.item.id), '已恢复修改前的状态。')
              }
            >
              {busy ? (
                <>
                  <LoaderCircle className="spinning" />
                  正在处理
                </>
              ) : (
                confirmLabel
              )}
            </button>
          </div>
        </Dialog>
      )}
      {adding && (
        <AddDialog
          initialScope={scope}
          busy={busy}
          operationError={notice?.kind === 'error' ? notice.text : ''}
          onClose={() => setAdding(false)}
          onAdd={(menu) =>
            operate(() => api.create(menu), '自定义菜单已添加到当前用户，可在操作历史中撤销。')
          }
        />
      )}
      {help && (
        <Dialog title="使用说明" onClose={() => setHelp(false)}>
          <p className="modal-description">
            管理本机常见位置的传统右键菜单，所有操作记录保存在本地。
          </p>
          <ul className="help-list">
            <li>第一版扫描 64 位的通用文件、文件夹、空白处和磁盘注册位置。</li>
            <li>已适配的普通命令可以停用、启用，并在操作历史中恢复。</li>
            <li>程序扩展、子菜单、系统基础项目暂时只提供查看。</li>
            <li>特定扩展名或 ProgID 的菜单、Windows 11 新版菜单暂不覆盖。</li>
            <li>来源名称按执行命令推断，未加载第三方扩展代码。</li>
            <li>修改后先重新打开右键菜单；缓存未更新时可自行重启资源管理器。</li>
          </ul>
          <div className="modal-actions">
            <button className="button primary" onClick={() => setHelp(false)}>
              知道了
            </button>
          </div>
        </Dialog>
      )}
    </>
  );
}

function AddDialog({
  initialScope,
  busy,
  operationError,
  onClose,
  onAdd,
}: {
  initialScope: Scope;
  busy: boolean;
  operationError: string;
  onClose: () => void;
  onAdd: (menu: NewMenu) => Promise<void>;
}) {
  const [menu, setMenu] = useState<NewMenu>({
    name: '',
    scope: ['file', 'folder', 'background', 'drive'].includes(initialScope)
      ? (initialScope as NewMenu['scope'])
      : 'background',
    program: '',
    arguments: '"{path}"',
  });
  const [error, setError] = useState('');
  const [picking, setPicking] = useState(false);
  const submit = async (event: FormEvent) => {
    event.preventDefault();
    setError('');
    await onAdd(menu);
  };
  async function browse() {
    setPicking(true);
    setError('');
    try {
      const path = await open({
        multiple: false,
        directory: false,
        filters: [{ name: '程序或 PowerShell 脚本', extensions: ['exe', 'ps1'] }],
      });
      if (typeof path === 'string') setMenu((m) => ({ ...m, program: path }));
    } catch (e) {
      setError(String(e));
    } finally {
      setPicking(false);
    }
  }
  return (
    <Dialog title="添加自定义菜单" busy={busy || picking} onClose={onClose}>
      <p className="modal-description">
        把常用程序或 PowerShell 脚本放进右键菜单，直接处理选中的路径。
      </p>
      <form onSubmit={submit}>
        <label className="field">
          <span>菜单名称</span>
          <input
            value={menu.name}
            maxLength={80}
            required
            disabled={busy}
            placeholder="例如：用编辑器打开"
            onChange={(e) => setMenu({ ...menu, name: e.target.value })}
          />
        </label>
        <label className="field">
          <span>显示位置</span>
          <select
            value={menu.scope}
            disabled={busy}
            onChange={(e) => setMenu({ ...menu, scope: e.target.value as NewMenu['scope'] })}
          >
            {scopes
              .filter((s) => ['file', 'folder', 'background', 'drive'].includes(s.id))
              .map((s) => (
                <option key={s.id} value={s.id}>
                  {s.name}
                </option>
              ))}
          </select>
        </label>
        <label className="field">
          <span>程序或脚本路径</span>
          <div className="path-picker">
            <input
              value={menu.program}
              required
              disabled={busy}
              placeholder="请选择实际存在的 .exe 或 .ps1 文件"
              onChange={(e) => setMenu({ ...menu, program: e.target.value })}
            />
            <button
              type="button"
              className="button secondary"
              disabled={busy || picking}
              onClick={browse}
            >
              选择文件
            </button>
          </div>
          <small>第一版支持 .exe 程序与 .ps1 脚本。添加时不会执行命令。</small>
        </label>
        <label className="field">
          <span>
            启动参数 <span className="field-optional">（可选）</span>
          </span>
          <input
            value={menu.arguments}
            disabled={busy}
            onChange={(e) => setMenu({ ...menu, arguments: e.target.value })}
          />
          <small>
            使用 {'{path}'}{' '}
            代表当前路径，并保留两侧引号以支持含空格的路径。脚本遵循系统现有的执行策略。
          </small>
        </label>
        <div className="confirmation-note add-note">
          <Info />
          <span>添加到当前用户，不需要管理员权限。可以在操作历史中撤销添加。</span>
        </div>
        {(error || operationError) && (
          <p className="form-error" role="alert">
            {error || operationError}
          </p>
        )}
        <div className="modal-actions">
          <button type="button" className="button secondary" disabled={busy} onClick={onClose}>
            取消
          </button>
          <button className="button primary" type="submit" disabled={busy || picking}>
            {busy ? '正在添加…' : '添加菜单'}
          </button>
        </div>
      </form>
    </Dialog>
  );
}
