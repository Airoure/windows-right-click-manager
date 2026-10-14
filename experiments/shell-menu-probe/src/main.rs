//! Read-only feasibility probe. Generates a Shell menu but never displays it or invokes a command.
use serde::Serialize;
use windows::{
    core::{Interface, PSTR, PWSTR},
    Win32::{
        Foundation::{HWND, LPARAM, WPARAM},
        System::{Com::{CoCreateInstance, CoInitializeEx, CoUninitialize, IServiceProvider, CLSCTX_LOCAL_SERVER, COINIT_APARTMENTTHREADED}, Variant::VARIANT},
        UI::{
            Shell::{IContextMenu, IContextMenu2, IContextMenu3, IShellView, IShellWindows, IShellBrowser, SHGetDesktopFolder, ShellWindows, SID_STopLevelBrowser, SWC_DESKTOP, SWFO_NEEDDISPATCH, CMF_NORMAL, GCS_VERBW, SVGIO_BACKGROUND},
            WindowsAndMessaging::{CreatePopupMenu, DestroyMenu, GetMenuItemCount, GetMenuItemInfoW, HMENU, MENUITEMINFOW, MIIM_FTYPE, MIIM_ID, MIIM_STATE, MIIM_STRING, MIIM_SUBMENU, MFT_SEPARATOR, WM_INITMENUPOPUP},
        },
    },
};

struct Apartment;
impl Drop for Apartment { fn drop(&mut self) { unsafe { CoUninitialize(); } } }
struct Menu(HMENU);
impl Drop for Menu { fn drop(&mut self) { let _ = unsafe { DestroyMenu(self.0) }; } }

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Item { text: String, command_id: u32, verb: String, state: u32, separator: bool, children: Vec<Item> }

unsafe fn read_menu(menu: HMENU, context: &IContextMenu, depth: usize) -> windows::core::Result<Vec<Item>> {
    let mut items = Vec::new();
    if depth > 2 { return Ok(items); }
    for index in 0..GetMenuItemCount(Some(menu)).clamp(0, 512) as u32 {
        let mut text = vec![0u16; 2048];
        let mut info = MENUITEMINFOW {
            cbSize: std::mem::size_of::<MENUITEMINFOW>() as u32,
            fMask: MIIM_STRING | MIIM_SUBMENU | MIIM_ID | MIIM_FTYPE | MIIM_STATE,
            dwTypeData: PWSTR(text.as_mut_ptr()), cch: text.len() as u32,
            ..Default::default()
        };
        GetMenuItemInfoW(menu, index, true, &mut info)?;
        let end = text.iter().position(|c| *c == 0).unwrap_or(text.len());
        let mut verb = vec![0u16; 512];
        if info.wID > 0 && info.wID < 0x8000 {
            if context.GetCommandString((info.wID - 1) as usize, GCS_VERBW, None, PSTR(verb.as_mut_ptr().cast()), verb.len() as u32).is_err() { verb.fill(0); }
        }
        let verb_end = verb.iter().position(|c| *c == 0).unwrap_or(verb.len());
        let children = if !info.hSubMenu.0.is_null() {
            if let Ok(handler) = context.cast::<IContextMenu3>() { let _ = handler.HandleMenuMsg2(WM_INITMENUPOPUP, WPARAM(info.hSubMenu.0 as usize), LPARAM(index as isize), None); }
            else if let Ok(handler) = context.cast::<IContextMenu2>() { let _ = handler.HandleMenuMsg(WM_INITMENUPOPUP, WPARAM(info.hSubMenu.0 as usize), LPARAM(index as isize)); }
            read_menu(info.hSubMenu, context, depth + 1)?
        } else { Vec::new() };
        items.push(Item { text: String::from_utf16_lossy(&text[..end]), command_id: info.wID, verb: String::from_utf16_lossy(&verb[..verb_end]), state: info.fState.0, separator: info.fType.contains(MFT_SEPARATOR), children });
    }
    Ok(items)
}

fn main() -> windows::core::Result<()> {
    unsafe {
        CoInitializeEx(None, COINIT_APARTMENTTHREADED).ok()?;
        let _apartment = Apartment;
        let desktop = SHGetDesktopFolder()?;
        let mode = std::env::args().nth(1).unwrap_or_else(|| "folder".into());
        let context = if mode == "active" {
            let windows: IShellWindows = CoCreateInstance(&ShellWindows, None, CLSCTX_LOCAL_SERVER)?;
            eprintln!("active: ShellWindows connected");
            let location = VARIANT::from(0i32);
            let empty = VARIANT::default();
            let mut hwnd = 0;
            let dispatch = windows.FindWindowSW(&location, &empty, SWC_DESKTOP, &mut hwnd, SWFO_NEEDDISPATCH)?;
            eprintln!("active: desktop located");
            let provider: IServiceProvider = dispatch.cast()?;
            eprintln!("active: service provider connected");
            let browser: IShellBrowser = provider.QueryService(&SID_STopLevelBrowser)?;
            eprintln!("active: browser connected");
            let view = browser.QueryActiveShellView()?;
            eprintln!("active: current view connected");
            view.GetItemObject::<IContextMenu>(SVGIO_BACKGROUND)?
        } else if mode == "view" {
            let view: IShellView = desktop.CreateViewObject(HWND::default())?;
            view.GetItemObject::<IContextMenu>(SVGIO_BACKGROUND)?
        } else { desktop.CreateViewObject::<IContextMenu>(HWND::default())? };
        let menu = Menu(CreatePopupMenu()?);
        context.QueryContextMenu(menu.0, 0, 1, 0x7fff, CMF_NORMAL).ok()?;
        println!("{}", serde_json::to_string_pretty(&read_menu(menu.0, &context, 0)?).unwrap());
    }
    Ok(())
}
