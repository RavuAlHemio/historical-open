use windows::core::{PWSTR, w};
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::Threading::{GetStartupInfoW, STARTF_USESHOWWINDOW, STARTUPINFOW};
use windows::Win32::UI::Controls::Dialogs::{
    CommDlgExtendedError, GetOpenFileNameW, OFN_ALLOWMULTISELECT, OFN_ENABLEHOOK, OFN_EXPLORER,
    OFN_EX_NOPLACESBAR, OPENFILENAMEW, OPEN_FILENAME_FLAGS, OPEN_FILENAME_FLAGS_EX,
};
use windows::Win32::UI::WindowsAndMessaging::{
    BN_CLICKED, BS_PUSHBUTTON, CreateWindowExW, CW_USEDEFAULT, DefWindowProcW, DispatchMessageW,
    GetMessageW, GetWindowLongPtrW, GWLP_HINSTANCE, HMENU, IsDialogMessageW, MSG, PostQuitMessage,
    RegisterClassW, ShowWindow, SHOW_WINDOW_CMD, SW_SHOW, TranslateMessage, WINDOW_EX_STYLE,
    WINDOW_STYLE, WM_COMMAND, WM_CREATE, WM_DESTROY, WNDCLASSW, WS_CHILD, WS_OVERLAPPEDWINDOW,
    WS_TABSTOP, WS_VISIBLE,
};


unsafe extern "system" fn hook_procedure_that_returns_false(_window_handle: HWND, _message: u32, _wparam: WPARAM, _lparam: LPARAM) -> usize {
    0
}


fn show_open_dialog(parent_window: HWND, flags: OPEN_FILENAME_FLAGS, ex_flags: OPEN_FILENAME_FLAGS_EX) {
    let mut file_buffer = [0u16; 260];

    let mut open_file_name = OPENFILENAMEW::default();
    open_file_name.lStructSize = std::mem::size_of_val(&open_file_name).try_into().unwrap();
    open_file_name.hwndOwner = parent_window;
    open_file_name.lpstrFilter = w!("All Files (*.*)\0*.*\0\0");
    open_file_name.lpstrFile = PWSTR(file_buffer.as_mut_ptr());
    open_file_name.nMaxFile = file_buffer.len().try_into().unwrap();
    open_file_name.lpstrFileTitle = PWSTR(std::ptr::null_mut());
    open_file_name.Flags = flags;
    open_file_name.lpfnHook = Some(hook_procedure_that_returns_false);
    open_file_name.FlagsEx = ex_flags;

    let result = unsafe { GetOpenFileNameW(&mut open_file_name) };
    if !result.as_bool() {
        let extended_error = unsafe { CommDlgExtendedError() };
        println!("extended error: {:?}", extended_error);
    }
}


unsafe extern "system" fn window_proc(window_handle: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    //println!("{} {}", message, window_message_name(message));

    if message == WM_CREATE {
        let buttons = [
            w!("Windows 3.1"),
            w!("Windows 95"),
            w!("Windows 2000/ME"),
            w!("Windows Vista"),
        ];
        let module_instance_handle = HINSTANCE(unsafe { GetWindowLongPtrW(window_handle, GWLP_HINSTANCE) });
        for (i, button_label) in buttons.iter().enumerate() {
            let y_pos = i32::try_from(i).unwrap() * 100;
            let button_handle = unsafe {
                CreateWindowExW(
                    WINDOW_EX_STYLE(0),
                    w!("BUTTON"),
                    *button_label,
                    WINDOW_STYLE(BS_PUSHBUTTON.try_into().unwrap()) | WS_CHILD | WS_TABSTOP | WS_VISIBLE,
                    10, 10 + y_pos, 300, 90,
                    window_handle,
                    HMENU((i + 1).try_into().unwrap()),
                    module_instance_handle,
                    None,
                )
            };
            if button_handle.0 == 0 {
                panic!("failed to create button with index {}: {}", i, windows::core::Error::from_win32());
            }
        }
        LRESULT(0)
    } else if message == WM_DESTROY {
        PostQuitMessage(0);
        LRESULT(0)
    } else if message == WM_COMMAND {
        let notification_code = ((wparam.0 >> 16) & 0xFFFF) as u32;
        let control_identifier = ((wparam.0 >> 0) & 0xFFFF) as u32;

        if notification_code == BN_CLICKED {
            match control_identifier {
                1 => show_open_dialog(window_handle, OFN_ALLOWMULTISELECT, OPEN_FILENAME_FLAGS_EX(0)),
                2 => show_open_dialog(window_handle, OFN_ALLOWMULTISELECT | OFN_EXPLORER | OFN_ENABLEHOOK, OFN_EX_NOPLACESBAR),
                3 => show_open_dialog(window_handle, OFN_ALLOWMULTISELECT | OFN_EXPLORER | OFN_ENABLEHOOK, OPEN_FILENAME_FLAGS_EX(0)),
                4 => show_open_dialog(window_handle, OPEN_FILENAME_FLAGS(0), OPEN_FILENAME_FLAGS_EX(0)),
                _ => {},
            }
        }

        LRESULT(0)
    } else {
        DefWindowProcW(window_handle, message, wparam, lparam)
    }
}


fn main() {
    // get startup info
    let mut startup_info = STARTUPINFOW::default();
    unsafe {
        GetStartupInfoW(&mut startup_info)
    };
    let show_window_cmd = if startup_info.dwFlags.contains(STARTF_USESHOWWINDOW) {
        SHOW_WINDOW_CMD(startup_info.wShowWindow.into())
    } else {
        SW_SHOW
    };

    // set up the main window
    let class_name = w!("HistoricalOpenWindowClass");
    let module_handle = unsafe { GetModuleHandleW(None) }
        .expect("failed to obtain module handle");
    let module_instance_handle: HINSTANCE = module_handle.into();

    let mut window_class = WNDCLASSW::default();
    window_class.lpfnWndProc = Some(window_proc);
    window_class.hInstance = module_instance_handle;
    window_class.lpszClassName = class_name;
    let ret = unsafe { RegisterClassW(&window_class) };
    if ret == 0 {
        panic!("failed to register window class: {}", windows::core::Error::from_win32());
    }

    let window_name = w!("Open File Dialogs");
    let window_handle = unsafe {
        CreateWindowExW(
            WINDOW_EX_STYLE(0),
            class_name,
            window_name,
            WS_OVERLAPPEDWINDOW,
            CW_USEDEFAULT, CW_USEDEFAULT, CW_USEDEFAULT, CW_USEDEFAULT,
            None,
            None,
            module_instance_handle,
            None,
        )
    };
    if window_handle.0 == 0 {
        panic!("failed to create window: {}", windows::core::Error::from_win32());
    }

    let _ = unsafe { ShowWindow(window_handle, show_window_cmd) };

    // event pump
    let mut message = MSG::default();
    loop {
        let get_message = unsafe { GetMessageW(&mut message, None, 0, 0) };
        if get_message.0 == 0 {
            break;
        } else if get_message.0 == -1 {
            panic!("failed to obtain window message: {}", windows::core::Error::from_win32());
        }
        let is_dialog_message = unsafe { IsDialogMessageW(window_handle, &message) };
        if !is_dialog_message.as_bool() {
            let _ = unsafe { TranslateMessage(&message) };
            let _ = unsafe { DispatchMessageW(&message) };
        }
    }
}
