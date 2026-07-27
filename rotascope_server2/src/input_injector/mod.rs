use rotascope_core::shared::protocol::TouchEvent;

#[cfg(windows)]
pub fn inject_input(event: &TouchEvent, screen_width: u32, screen_height: u32) -> anyhow::Result<()> {
    use windows::Win32::UI::Input::KeyboardAndMouse::{
        SendInput, INPUT, INPUT_0, INPUT_MOUSE, MOUSEINPUT, MOUSE_EVENT_FLAGS,
        MOUSEEVENTF_MOVE, MOUSEEVENTF_ABSOLUTE, MOUSEEVENTF_LEFTDOWN,
        MOUSEEVENTF_LEFTUP,
    };

    unsafe {
        match event {
            TouchEvent::Move { x, y } => {
                let input = INPUT {
                    r#type: INPUT_MOUSE,
                    Anonymous: INPUT_0 {
                        mi: MOUSEINPUT {
                            dx: ((x / screen_width as f32) * 65535.0) as i32,
                            dy: ((y / screen_height as f32) * 65535.0) as i32,
                            mouseData: 0,
                            dwFlags: MOUSEEVENTF_MOVE | MOUSEEVENTF_ABSOLUTE,
                            time: 0,
                            dwExtraInfo: 0,
                        },
                    },
                };
                _ = SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
            }
            TouchEvent::Down { x, y } => {
                let input = INPUT {
                    r#type: INPUT_MOUSE,
                    Anonymous: INPUT_0 {
                        mi: MOUSEINPUT {
                            dx: ((x / screen_width as f32) * 65535.0) as i32,
                            dy: ((y / screen_height as f32) * 65535.0) as i32,
                            mouseData: 0,
                            dwFlags: MOUSEEVENTF_LEFTDOWN | MOUSEEVENTF_ABSOLUTE,
                            time: 0,
                            dwExtraInfo: 0,
                        },
                    },
                };
                _ = SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
            }
            TouchEvent::Up { x, y } => {
                let input = INPUT {
                    r#type: INPUT_MOUSE,
                    Anonymous: INPUT_0 {
                        mi: MOUSEINPUT {
                            dx: ((x / screen_width as f32) * 65535.0) as i32,
                            dy: ((y / screen_height as f32) * 65535.0) as i32,
                            mouseData: 0,
                            dwFlags: MOUSEEVENTF_LEFTUP | MOUSEEVENTF_ABSOLUTE,
                            time: 0,
                            dwExtraInfo: 0,
                        },
                    },
                };
                _ = SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
            }
            TouchEvent::Scroll { delta_x, delta_y } => {
                if *delta_y != 0.0 {
                    let input = INPUT {
                        r#type: INPUT_MOUSE,
                        Anonymous: INPUT_0 {
                            mi: MOUSEINPUT {
                                dx: 0,
                                dy: 0,
                                mouseData: (*delta_y * 120.0) as u32,
                                dwFlags: MOUSE_EVENT_FLAGS(2048u32), // MOUSEEVENTF_WHEEL
                                time: 0,
                                dwExtraInfo: 0,
                            },
                        },
                    };
                    _ = SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
                }
                if *delta_x != 0.0 {
                    let input = INPUT {
                        r#type: INPUT_MOUSE,
                        Anonymous: INPUT_0 {
                            mi: MOUSEINPUT {
                                dx: 0,
                                dy: 0,
                                mouseData: (*delta_x * 120.0) as u32,
                                dwFlags: MOUSE_EVENT_FLAGS(4096u32), // MOUSEEVENTF_HWHEEL
                                time: 0,
                                dwExtraInfo: 0,
                            },
                        },
                    };
                    _ = SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
                }
            }
        }
    }

    Ok(())
}
