#include "Driver.h"
#include <iddcx.h>

extern "C"
NTSTATUS CreateVirtualMonitor()
{
    DbgPrint("[RotaScope] Creating Virtual Monitor\n");

    return STATUS_SUCCESS;
}

// Monitor arrival callback
void EvtMonitorArrival(IDDCX_ADAPTER Adapter, IDDCX_MONITOR Monitor)
{
    DbgPrint("[RotaScope] Monitor arrival detected\n");

    // Assign swap chain for the monitor
    IDDCX_SWAPCHAIN swapChain = nullptr;
    NTSTATUS status = IddCxMonitorAssignSwapChain(
        Monitor,
        nullptr,
        &swapChain
    );

    if (NT_SUCCESS(status)) {
        DbgPrint("[RotaScope] Swap chain assigned to monitor\n");
    }
}
