use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TouchEvent {
    Move { x: f32, y: f32 },
    Down { x: f32, y: f32 },
    Up { x: f32, y: f32 },
    Scroll { delta_x: f32, delta_y: f32 },
}

#[cfg(windows)]
pub fn inject_input(event: &TouchEvent, screen_width: u32, screen_height: u32) -> anyhow::Result<()> {
    use windows::Win32::UI::WindowsAndMessaging::{
        SendInput, INPUT, PT_MOUSE, MOUSEINPUT, MOUSEEVENTF_MOVE,
        MOUSEEVENTF_ABSOLUTE, MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP,
        MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP
    };

    unsafe {
        match event {
            TouchEvent::Move { x, y } => {
                let input = INPUT {
                    r#type: INPUT_MOUSE,
                    Anonymous: std::mem::transmute(MOUSEINPUT {
                        dx: ((x / screen_width as f32) * 65535.0) as i32,
                        dy: ((y / screen_height as f32) * 65535.0) as i32,
                        mouseData: 0,
                        dwFlags: MOUSEEVENTF_MOVE | MOUSEEVENTF_ABSOLUTE,
                        time: 0,
                        dwExtraInfo: 0,
                    }),
                };
                SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
            }
            TouchEvent::Down { x: _, y: _ } => {
                let input = INPUT {
                    r#type: INPUT_MOUSE,
                    Anonymous: std::mem::transmute(MOUSEINPUT {
                        dx: 0,
                        dy: 0,
                        mouseData: 0,
                        dwFlags: MOUSEEVENTF_LEFTDOWN,
                        time: 0,
                        dwExtraInfo: 0,
                    }),
                };
                SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
            }
            TouchEvent::Up { x: _, y: _ } => {
                let input = INPUT {
                    r#type: INPUT_MOUSE,
                    Anonymous: std::mem::transmute(MOUSEINPUT {
                        dx: 0,
                        dy: 0,
                        mouseData: 0,
                        dwFlags: MOUSEEVENTF_LEFTUP,
                        time: 0,
                        dwExtraInfo: 0,
                    }),
                };
                SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
            }
            _ => {} // 其他事件暂不处理
        }
    }

    Ok(())
}