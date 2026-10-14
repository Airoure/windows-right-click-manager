import { ChevronRight, MousePointer2, RefreshCw, Info } from 'lucide-react';
import type { CapturedItem, CapturedMenu, MenuEntry, Scope } from './types';

// Captions are only a navigation hint. They are not proof of registry ownership.
function caption(text: string) {
  return text
    .replace(/&&/g, '\u0000')
    .replace(/&/g, '')
    .replace(/\u0000/g, '&')
    .split('\t')[0]
    .trim();
}
function related(item: CapturedItem, entries: MenuEntry[], scope: Scope) {
  return entries.filter(
    (e) =>
      e.enabled &&
      e.kind !== 'extension' &&
      (scope === 'all' ||
        e.scope === scope ||
        (['file', 'folder', 'drive'].includes(scope) && e.scope === 'filesystem')) &&
      caption(e.name) === caption(item.name),
  );
}
function Rows({
  items,
  entries,
  scope,
  onSelect,
  nested = false,
}: {
  items: CapturedItem[];
  entries: MenuEntry[];
  scope: Scope;
  onSelect: (id: string) => void;
  nested?: boolean;
}) {
  return (
    <ol className={`actual-rows ${nested ? 'nested' : ''}`}>
      {items.map((item, index) => {
        if (item.separator) return <li key={index} className="actual-separator" role="separator" />;
        const matches = nested ? [] : related(item, entries, scope);
        return (
          <li key={index}>
            <div className={`actual-row ${item.disabled ? 'unavailable' : ''}`}>
              <span className="actual-name">{item.name}</span>
              <span className="actual-shortcut">{item.shortcut}</span>
              {item.disabled && <span className="actual-state">当前不可用</span>}
              {item.hasSubmenu && <ChevronRight aria-label="子菜单" />}
              {matches.length === 1 && (
                <button className="actual-link" onClick={() => onSelect(matches[0].id)}>
                  核对同名注册项
                </button>
              )}
            </div>
            {item.children.length > 0 ? (
              <Rows
                items={item.children}
                entries={entries}
                scope={scope}
                onSelect={onSelect}
                nested
              />
            ) : (
              item.hasSubmenu && (
                <p className="unread-submenu">子菜单未读取 · 下次读取时将鼠标移到此项展开</p>
              )
            )}
          </li>
        );
      })}
    </ol>
  );
}
export function ActualMenu({
  capture,
  stale,
  reading,
  enabled,
  entries,
  scope,
  onRead,
  onSelect,
}: {
  capture?: CapturedMenu;
  stale: boolean;
  reading: boolean;
  enabled: boolean;
  entries: MenuEntry[];
  scope: Scope;
  onRead: () => void;
  onSelect: (id: string) => void;
}) {
  return (
    <section className="actual-panel" aria-label="实际右键菜单">
      <div className="actual-toolbar">
        <div>
          <strong>{capture ? '你刚才看到的菜单' : '从一次真实右键开始'}</strong>
          <p>
            {capture
              ? `读取于 ${new Date(capture.capturedAt).toLocaleTimeString('zh-CN')} · 按弹出顺序显示`
              : '窗口会暂时最小化。右键打开菜单，展开需要的子菜单，再按 Esc 关闭。'}
          </p>
        </div>
        <button className="button primary" disabled={!enabled || reading} onClick={onRead}>
          <RefreshCw className={reading ? 'spinning' : ''} />
          {reading ? '正在等待读取' : capture ? '重新读取实际菜单' : '读取实际菜单'}
        </button>
      </div>
      {stale && (
        <div className="notice">
          <Info />
          <span>菜单设置已改变。这是修改前的记录，请重新读取确认实际效果。</span>
        </div>
      )}
      {capture?.warnings.map((warning, i) => (
        <div className="notice" key={i}>
          <Info />
          <span>{warning}</span>
        </div>
      ))}
      {capture ? (
        <Rows items={capture.items} entries={entries} scope={scope} onSelect={onSelect} />
      ) : (
        <div className="empty-state actual-empty">
          <MousePointer2 />
          <strong>这里显示实际弹出的菜单</strong>
          <span>包括程序动态添加的项目、分隔线，以及当时的灰色状态。</span>
          <span>请在当前选择的位置右键；文件类型不同，结果也可能不同。</span>
        </div>
      )}
      <div className="list-footer">
        <Info />
        <span>
          此处保存本次读取的文字与层级，不执行菜单命令。灰色表示当时不可用；停用与恢复请在“注册项”或“操作历史”中查看。图标暂未读取。
        </span>
      </div>
    </section>
  );
}
