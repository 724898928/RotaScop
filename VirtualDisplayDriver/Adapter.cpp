#include "Driver.h"
#include <iddcx.h>

NTSTATUS CreateVirtualAdapter(PDRIVER_OBJECT DriverObject)
{
    DbgPrint("[RotaScope] Creating Virtual GPU Adapter\n");

    // Initialize IDD adapter
    IDDCX_ADAPTER_INIT init = {};
    init.Size = sizeof(init);

    // Set adapter capabilities
    IDDCX_ADAPTER_CAPS caps = {};
    caps.Size = sizeof(caps);
    caps.MaxMonitorsSupported = 4;
    caps.EndPointDiagnostics = nullptr;

    // Create the adapter
    IDDCX_ADAPTER adapter = nullptr;
    NTSTATUS status = IddCxAdapterCreate(
        DriverObject,
        &init,
        &caps,
        &adapter
    );

    if (!NT_SUCCESS(status)) {
        DbgPrint("[RotaScope] Failed to create adapter: 0x%X\n", status);
        return status;
    }

    DbgPrint("[RotaScope] Virtual adapter created successfully\n");

    // Create a monitor for this adapter
    CreateMonitor(adapter);

    return STATUS_SUCCESS;
}
