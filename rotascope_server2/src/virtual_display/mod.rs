use anyhow::Result;
use tracing::info;
#[cfg(not(windows))]
use tracing::warn;

pub struct VirtualDisplay {
    pub width: u32,
    pub height: u32,
    pub refresh_rate: u32,
}

impl VirtualDisplay {
    pub fn new(width: u32, height: u32, refresh_rate: u32) -> Self {
        Self {
            width,
            height,
            refresh_rate,
        }
    }
}

pub struct VirtualDisplayManager {
    pub displays: Vec<VirtualDisplay>,
}

impl VirtualDisplayManager {
    pub fn new() -> Self {
        Self { displays: Vec::new() }
    }

    pub fn add_display(&mut self, width: u32, height: u32, refresh_rate: u32) {
        self.displays.push(VirtualDisplay::new(width, height, refresh_rate));
    }

    pub fn create_virtual_display(&self, display_id: u32) -> Result<()> {
        let disp = self
            .displays
            .get(display_id as usize)
            .ok_or_else(|| anyhow::anyhow!("Virtual display {} not found", display_id))?;

        info!(
            "Creating virtual display {}: {}x{}@{}Hz",
            display_id, disp.width, disp.height, disp.refresh_rate
        );

        #[cfg(windows)]
        {
            create_windows_virtual_display(display_id, disp.width, disp.height, disp.refresh_rate)?;
        }

        #[cfg(not(windows))]
        {
            warn!("Virtual display creation is only supported on Windows");
        }

        Ok(())
    }
}

#[cfg(windows)]
fn create_windows_virtual_display(
    display_id: u32,
    width: u32,
    height: u32,
    refresh_rate: u32,
) -> Result<()> {
    use std::ffi::OsStr;
    use std::mem;
    use std::os::windows::prelude::OsStrExt;
    use std::ptr;
    use windows::Win32::Graphics::Gdi::{
        ChangeDisplaySettingsExW, CDS_NORESET, CDS_UPDATEREGISTRY, DEVMODEW,
        DISP_CHANGE_SUCCESSFUL,
    };

    let device_name: Vec<u16> = OsStr::new(&format!("\\\\.\\DISPLAY{}", display_id + 1))
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    unsafe {
        let mut devmode: DEVMODEW = mem::zeroed();
        devmode.dmSize = mem::size_of::<DEVMODEW>() as u16;
        devmode.dmFields = windows::Win32::Graphics::Gdi::DM_POSITION
            | windows::Win32::Graphics::Gdi::DM_PELSWIDTH
            | windows::Win32::Graphics::Gdi::DM_PELSHEIGHT
            | windows::Win32::Graphics::Gdi::DM_DISPLAYFREQUENCY;
        devmode.dmPelsWidth = width;
        devmode.dmPelsHeight = height;
        devmode.Anonymous1.Anonymous2.dmPosition.x = 0;
        devmode.Anonymous1.Anonymous2.dmPosition.y = 0;
        devmode.dmDisplayFrequency = refresh_rate as u32;

        let result = ChangeDisplaySettingsExW(
            windows::core::PCWSTR(device_name.as_ptr()),
            Some(&devmode),
            None,
            CDS_UPDATEREGISTRY | CDS_NORESET,
            Some(ptr::null_mut()),
        );

        if result == DISP_CHANGE_SUCCESSFUL {
            info!("Virtual display {} created successfully", display_id);
            Ok(())
        } else {
            anyhow::bail!(
                "Failed to create virtual display: error code {}",
                result.0
            );
        }
    }
}
