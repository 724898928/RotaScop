use rotascope_core::shared::protocol::TouchEvent;

#[cfg(windows)]
pub fn inject_input(event: &TouchEvent, screen_width: u32, screen_height: u32) -> anyhow::Result<()> {
    use windows::Win32::UI::Input::KeyboardAndMouse::{
        SendInput, INPUT, INPUT_0, INPUT_MOUSE, MOUSEINPUT,
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
            TouchEvent::Down { x: _, y: _ } => {
                let input = INPUT {
                    r#type: INPUT_MOUSE,
                    Anonymous: INPUT_0 {
                        mi: MOUSEINPUT {
                            dx: 0,
                            dy: 0,
                            mouseData: 0,
                            dwFlags: MOUSEEVENTF_LEFTDOWN,
                            time: 0,
                            dwExtraInfo: 0,
                        },
                    },
                };
                _ = SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
            }
            TouchEvent::Up { x: _, y: _ } => {
                let input = INPUT {
                    r#type: INPUT_MOUSE,
                    Anonymous: INPUT_0 {
                        mi: MOUSEINPUT {
                            dx: 0,
                            dy: 0,
                            mouseData: 0,
                            dwFlags: MOUSEEVENTF_LEFTUP,
                            time: 0,
                            dwExtraInfo: 0,
                        },
                    },
                };
                _ = SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
            }
            _ => {}
        }
    }

    Ok(())
}
