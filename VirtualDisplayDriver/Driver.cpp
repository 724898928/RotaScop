#include "Driver.h"
#include "Shared.h"

// Global shared frame buffer
SharedFrame* g_SharedFrame = nullptr;

extern "C"
NTSTATUS DriverEntry(PDRIVER_OBJECT DriverObject, PUNICODE_STRING RegistryPath)
{
    UNREFERENCED_PARAMETER(RegistryPath);

    DbgPrint("[RotaScope] Virtual Display Driver Loaded\n");

    DriverObject->DriverUnload = DriverUnload;

    // Allocate shared frame buffer
    g_SharedFrame = (SharedFrame*)ExAllocatePool2(
        POOL_FLAG_NON_PAGED,
        sizeof(SharedFrame),
        'RTSD'
    );

    if (!g_SharedFrame) {
        DbgPrint("[RotaScope] Failed to allocate shared frame buffer\n");
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    RtlZeroMemory(g_SharedFrame, sizeof(SharedFrame));

    return InitIDD(DriverObject);
}

extern "C"
VOID DriverUnload(PDRIVER_OBJECT DriverObject)
{
    UNREFERENCED_PARAMETER(DriverObject);

    DbgPrint("[RotaScope] Driver Unloaded\n");

    if (g_SharedFrame) {
        ExFreePoolWithTag(g_SharedFrame, 'RTSD');
        g_SharedFrame = nullptr;
    }
}

extern "C"
NTSTATUS InitIDD(PDRIVER_OBJECT DriverObject)
{
    DbgPrint("[RotaScope] Initializing Indirect Display Driver\n");

    // Store pointer to driver object (used by adapter)
    DriverObject->DriverExtension->AddDevice = CreateVirtualAdapter;

    return STATUS_SUCCESS;
}
