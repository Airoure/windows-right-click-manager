import { useEffect, useRef, type ReactNode } from 'react';
import { createPortal } from 'react-dom';
import {
  CodeXml,
  File,
  Folder,
  GitBranch,
  HardDrive,
  LayoutGrid,
  PanelTop,
  Sparkles,
  Terminal,
  X,
} from 'lucide-react';
import type { MenuEntry, Scope } from './types';

export function ScopeIcon({ scope }: { scope: Scope }) {
  const Component = {
    all: LayoutGrid,
    desktop: PanelTop,
    file: File,
    filesystem: LayoutGrid,
    folder: Folder,
    background: PanelTop,
    drive: HardDrive,
  }[scope];
  return <Component aria-hidden="true" />;
}
export function AppIcon({ entry }: { entry: MenuEntry }) {
  const source = entry.source.toLowerCase();
  const type = source.includes('git')
    ? 'git'
    : source.includes('warp')
      ? 'warp'
      : source.includes('deveco')
        ? 'deveco'
        : source.includes('workbuddy')
          ? 'workbuddy'
          : source.includes('code')
            ? 'code'
            : 'terminal';
  const Component =
    type === 'git'
      ? GitBranch
      : type === 'workbuddy'
        ? Sparkles
        : ['code', 'deveco'].includes(type)
          ? CodeXml
          : Terminal;
  return (
    <span className={`app-icon ${type}`}>
      <Component aria-hidden="true" />
    </span>
  );
}
export function Dialog({
  title,
  children,
  onClose,
  drawer = false,
  busy = false,
}: {
  title: string;
  children: ReactNode;
  onClose: () => void;
  drawer?: boolean;
  busy?: boolean;
}) {
  const ref = useRef<HTMLDivElement>(null);
  const close = useRef(onClose);
  const pending = useRef(busy);
  close.current = onClose;
  pending.current = busy;
  useEffect(() => {
    const previous = document.activeElement as HTMLElement | null;
    const app = document.getElementById('app-content')!;
    app.inert = true;
    ref.current?.querySelector<HTMLElement>('input,button,select')?.focus();
    const keydown = (event: KeyboardEvent) => {
      const dialogs = document.querySelectorAll('[data-modal]');
      if (dialogs[dialogs.length - 1] !== ref.current) return;
      if (event.key === 'Escape' && !pending.current) {
        event.preventDefault();
        close.current();
      }
      if (event.key === 'Tab') {
        const focusable = [
          ...ref.current!.querySelectorAll<HTMLElement>(
            'button:not(:disabled),input,select,summary,[tabindex="0"]',
          ),
        ].filter((el) => el.offsetParent !== null);
        if (!focusable.length) {
          event.preventDefault();
          ref.current?.focus();
          return;
        }
        const first = focusable[0],
          last = focusable.at(-1)!;
        if (event.shiftKey && document.activeElement === first) {
          event.preventDefault();
          last.focus();
        } else if (!event.shiftKey && document.activeElement === last) {
          event.preventDefault();
          first.focus();
        }
      }
    };
    document.addEventListener('keydown', keydown);
    return () => {
      document.removeEventListener('keydown', keydown);
      queueMicrotask(() => {
        app.inert = document.querySelectorAll('[data-modal]').length > 0;
        if (previous?.isConnected) previous.focus();
        else if (!app.inert) document.querySelector<HTMLElement>('.search input')?.focus();
      });
    };
  }, []);
  return createPortal(
    <div
      className={drawer ? 'drawer-backdrop' : 'modal-backdrop'}
      onMouseDown={(event) => {
        if (event.target === event.currentTarget && !busy) onClose();
      }}
    >
      <div
        ref={ref}
        data-modal
        role="dialog"
        aria-modal="true"
        aria-label={title}
        className={drawer ? 'detail-panel' : 'modal wide'}
        tabIndex={-1}
      >
        {drawer ? (
          <button
            className="icon-button detail-close"
            aria-label="关闭菜单详情"
            disabled={busy}
            onClick={onClose}
          >
            <X />
          </button>
        ) : (
          <div className="modal-top">
            <h2>{title}</h2>
            <button
              className="icon-button"
              aria-label="关闭对话框"
              disabled={busy}
              onClick={onClose}
            >
              <X />
            </button>
          </div>
        )}
        {children}
      </div>
    </div>,
    document.body,
  );
}
