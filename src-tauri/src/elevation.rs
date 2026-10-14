use crate::{model::*, registry};
use std::{
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, Read, Write},
    os::windows::io::FromRawHandle,
    time::{Duration, Instant},
};
use windows_sys::Win32::{
    Foundation::*,
    Storage::FileSystem::*,
    System::{Pipes::*, Threading::*},
    UI::{Shell::*, WindowsAndMessaging::*},
};

fn wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(Some(0)).collect()
}
fn io_error() -> String {
    format!("管理员操作失败：{}", std::io::Error::last_os_error())
}
struct Process(HANDLE);
impl Drop for Process {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe {
                CloseHandle(self.0);
            }
        }
    }
}

// This elevated entry point can change only LegacyDisable on supported HKLM verbs.
// It never accepts a filesystem output path, executable command, or arbitrary registry value name.
pub fn helper(token: &str) -> Result<(), String> {
    if uuid::Uuid::parse_str(token).is_err() {
        return Err("无效的操作通道。".into());
    }
    let name = format!("\\\\.\\pipe\\RightClickManager-{token}");
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(&name)
        .map_err(|e| e.to_string())?;
    let mut input = String::new();
    BufReader::new((&file).take(8_000_001))
        .read_line(&mut input)
        .map_err(|e| e.to_string())?;
    if input.len() > 8_000_000 {
        return Err("操作请求超出限制。".into());
    }
    let result = (|| {
        let plan: LegacyPlan = serde_json::from_str(&input).map_err(|e| e.to_string())?;
        if plan.target.hive != "LM" {
            return Err("管理员辅助进程只处理机器级普通菜单。".into());
        }
        plan.target.validate()?;
        registry::apply_legacy(&plan)
    })();
    let response = serde_json::to_vec(&result).map_err(|e| e.to_string())?;
    file.write_all(&response).map_err(|e| e.to_string())?;
    file.write_all(b"\n").map_err(|e| e.to_string())?;
    Ok(())
}

pub fn execute(plan: &LegacyPlan) -> Result<KeySnapshot, String> {
    if plan.target.hive == "CU" {
        return registry::apply_legacy(plan);
    }
    let token = uuid::Uuid::new_v4().to_string();
    let name = wide(&format!("\\\\.\\pipe\\RightClickManager-{token}"));
    let handle = unsafe {
        CreateNamedPipeW(
            name.as_ptr(),
            PIPE_ACCESS_DUPLEX | FILE_FLAG_FIRST_PIPE_INSTANCE,
            PIPE_TYPE_BYTE | PIPE_READMODE_BYTE | PIPE_NOWAIT | PIPE_REJECT_REMOTE_CLIENTS,
            1,
            65536,
            65536,
            0,
            std::ptr::null(),
        )
    };
    if handle == INVALID_HANDLE_VALUE {
        return Err(io_error());
    }
    let mut pipe = unsafe { File::from_raw_handle(handle) };
    let exe = wide(
        &std::env::current_exe()
            .map_err(|e| e.to_string())?
            .to_string_lossy(),
    );
    let args = wide(&format!("--elevated-helper {token}"));
    let verb = wide("runas");
    let mut launch: SHELLEXECUTEINFOW = unsafe { std::mem::zeroed() };
    launch.cbSize = std::mem::size_of::<SHELLEXECUTEINFOW>() as u32;
    launch.fMask = SEE_MASK_NOCLOSEPROCESS | SEE_MASK_NOASYNC;
    launch.lpVerb = verb.as_ptr();
    launch.lpFile = exe.as_ptr();
    launch.lpParameters = args.as_ptr();
    launch.nShow = SW_HIDE;
    if unsafe { ShellExecuteExW(&mut launch) } == 0 {
        let e = std::io::Error::last_os_error();
        return Err(if e.raw_os_error() == Some(1223) {
            "已取消管理员授权，未执行修改。".into()
        } else {
            format!("无法启动管理员辅助进程：{e}")
        });
    }
    let process = Process(launch.hProcess);
    if process.0.is_null() {
        return Err("无法确认管理员辅助进程身份。".into());
    }
    let pid = unsafe { GetProcessId(process.0) };
    let deadline = Instant::now() + Duration::from_secs(90);
    loop {
        let connected = unsafe { ConnectNamedPipe(handle, std::ptr::null_mut()) } != 0;
        let code = std::io::Error::last_os_error().raw_os_error();
        if connected || code == Some(ERROR_PIPE_CONNECTED as i32) {
            break;
        }
        if code != Some(ERROR_PIPE_LISTENING as i32) {
            return Err(io_error());
        }
        if Instant::now() > deadline
            || unsafe { WaitForSingleObject(process.0, 0) } == WAIT_OBJECT_0
        {
            return Err("管理员辅助进程未连接，操作未确认。请检查操作历史。".into());
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    let mut client_pid = 0;
    if unsafe { GetNamedPipeClientProcessId(handle, &mut client_pid) } == 0 || client_pid != pid {
        return Err("操作通道的进程身份不匹配。".into());
    }
    let mut request = serde_json::to_vec(plan).map_err(|e| e.to_string())?;
    request.push(b'\n');
    if request.len() > 8_000_000 {
        return Err("操作请求超出限制。".into());
    }
    let mut offset = 0;
    while offset < request.len() {
        match pipe.write(&request[offset..]) {
            Ok(0) => {}
            Ok(n) => offset += n,
            Err(e) if e.raw_os_error() == Some(ERROR_NO_DATA as i32) => {}
            Err(e) => return Err(format!("无法发送管理员操作：{e}")),
        }
        if Instant::now() > deadline {
            return Err("发送操作超时，请查看操作历史。".into());
        }
        if offset < request.len() {
            std::thread::sleep(Duration::from_millis(20));
        }
    }
    let mut response = Vec::new();
    let mut buffer = [0; 8192];
    loop {
        match pipe.read(&mut buffer) {
            Ok(0) => {}
            Ok(n) => {
                response.extend_from_slice(&buffer[..n]);
                if response.contains(&b'\n') {
                    break;
                }
            }
            Err(e) if e.raw_os_error() == Some(ERROR_NO_DATA as i32) => {}
            Err(e) => return Err(format!("管理员操作结果未确认：{e}。恢复记录已保留。")),
        }
        if response.len() > 8_000_000 || Instant::now() > deadline {
            return Err("管理员操作结果未确认，请查看操作历史。".into());
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    serde_json::from_slice::<Result<KeySnapshot, String>>(&response)
        .map_err(|e| format!("无法解析管理员操作结果：{e}"))?
}
