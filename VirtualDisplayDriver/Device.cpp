#include "Driver.h"
#include "Shared.h"
#include <iddcx.h>

// Supported monitor modes
IDDCX_MONITOR_MODE g_Modes[] = {
    { {1920, 1080}, 120, IDDCX_MONITOR_MODE_ORIGIN_DESKTOP },
    { {1280, 720}, 120, IDDCX_MONITOR_MODE_ORIGIN_DESKTOP },
    { {1920, 1080}, 60, IDDCX_MONITOR_MODE_ORIGIN_DESKTOP },
    { {1280, 720}, 60, IDDCX_MONITOR_MODE_ORIGIN_DESKTOP },
};

void CreateMonitor(IDDCX_ADAPTER Adapter)
{
    DbgPrint("[RotaScope] Creating virtual monitor\n");

    IDDCX_MONITOR_INFO info = {};
    info.Size = sizeof(info);
    info.MonitorDescription.Type = IDDCX_MONITOR_DESCRIPTION_TYPE_GENERIC;
    info.MonitorDescription.DataSize = 0;
    info.MonitorDescription.Data = nullptr;

    // Set monitor modes
    IDDCX_MONITOR_MODE modes[ARRAYSIZE(g_Modes)];
    for (UINT i = 0; i < ARRAYSIZE(g_Modes); i++) {
        modes[i] = g_Modes[i];
    }

    IDDCX_TARGET_MODE targetMode = {};
    targetMode.Size = sizeof(targetMode);
    targetMode.MonitorModeArray = modes;
    targetMode.MonitorModeCount = ARRAYSIZE(g_Modes);

    // Create monitor with IDD
    IDDCX_MONITOR monitor = nullptr;
    NTSTATUS status = IddCxMonitorCreate(
        Adapter,
        &info,
        &targetMode,
        nullptr,
        &monitor
    );

    if (NT_SUCCESS(status)) {
        DbgPrint("[RotaScope] Virtual monitor created successfully\n");

        // Assign default modes
        IDDCX_MONITOR_MODE defaultMode = modes[0];
        IddCxMonitorAssignSwapChain(monitor, nullptr, nullptr);
    } else {
        DbgPrint("[RotaScope] Failed to create virtual monitor: 0x%X\n", status);
    }
}
