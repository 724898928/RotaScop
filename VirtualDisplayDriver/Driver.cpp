#include "Driver.h"

static DEVICE_CONTEXT* g_GlobalDeviceContext = NULL;

DEVICE_CONTEXT*
GetGlobalDeviceContext(VOID)
{
    return g_GlobalDeviceContext;
}

extern "C"
NTSTATUS NTAPI
DriverEntry(
    _In_ PDRIVER_OBJECT  DriverObject,
    _In_ PUNICODE_STRING RegistryPath
)
{
    WDF_DRIVER_CONFIG config;
    NTSTATUS status;

    WDF_DRIVER_CONFIG_INIT(&config, EvtDriverDeviceAdd);

    status = WdfDriverCreate(
        DriverObject,
        RegistryPath,
        WDF_NO_OBJECT_ATTRIBUTES,
        &config,
        WDF_NO_HANDLE
    );

    if (!NT_SUCCESS(status))
    {
        return status;
    }

    DbgPrint("RotaScope VirtualDisplayDriver: DriverEntry OK\n");

    return STATUS_SUCCESS;
}

NTSTATUS
EvtDriverDeviceAdd(
    _In_ WDFDRIVER Driver,
    _Inout_ PWDFDEVICE_INIT DeviceInit
)
{
    WDF_OBJECT_ATTRIBUTES deviceAttributes;
    WDFDEVICE device;
    DEVICE_CONTEXT* deviceCtx;
    NTSTATUS status;

    UNREFERENCED_PARAMETER(Driver);

    WDF_OBJECT_ATTRIBUTES_INIT_CONTEXT_TYPE(&deviceAttributes, DEVICE_CONTEXT);
    deviceAttributes.EvtCleanupCallback = EvtDeviceContextCleanup;

    status = WdfDeviceCreate(&DeviceInit, &deviceAttributes, &device);

    if (!NT_SUCCESS(status))
    {
        return status;
    }

    deviceCtx = GetDeviceContext(device);
    deviceCtx->WdfDevice = device;
    deviceCtx->AdapterHandle = NULL;
    deviceCtx->MonitorHandle = NULL;
    deviceCtx->SwapChain = NULL;
    deviceCtx->NextSurfaceAvailable = NULL;
    deviceCtx->SharedFrame = NULL;
    deviceCtx->FrameEventHandle = NULL;
    deviceCtx->FrameEventObject = NULL;

    KeInitializeSpinLock(&deviceCtx->FrameLock);

    status = FrameInitialize(deviceCtx);

    if (!NT_SUCCESS(status))
    {
        return status;
    }

    g_GlobalDeviceContext = deviceCtx;

    // Initialize IddCx for this device
    status = IddCxDeviceInitialize(device);

    if (!NT_SUCCESS(status))
    {
        DbgPrint("RotaScope: IddCxDeviceInitialize failed 0x%X\n", status);
        return status;
    }

    // Start adapter initialization asynchronously
    {
        IDDCX_ADAPTER_CAPS adapterCaps = { 0 };
        IDARG_IN_ADAPTER_INIT adapterInit = { 0 };
        IDARG_OUT_ADAPTER_INIT adapterOut = { 0 };

        adapterCaps.Size = sizeof(adapterCaps);
        adapterCaps.MaxMonitorsSupported = 1;
        adapterCaps.Flags = IDDCX_ADAPTER_FLAGS_USE_SMALLEST_MODE;

        adapterInit.WdfDevice = device;
        adapterInit.pCaps = &adapterCaps;
        adapterInit.ObjectAttributes = NULL;

        status = IddCxAdapterInitAsync(&adapterInit, &adapterOut);

        if (!NT_SUCCESS(status))
        {
            DbgPrint("RotaScope: IddCxAdapterInitAsync failed 0x%X\n", status);
            return status;
        }

        deviceCtx->AdapterHandle = adapterOut.AdapterObject;
    }

    // Create IO queue for IOCTLs
    {
        WDFQUEUE queue;
        WDF_IO_QUEUE_CONFIG queueConfig;

        WDF_IO_QUEUE_CONFIG_INIT_DEFAULT_QUEUE(&queueConfig, WdfIoQueueDispatchSequential);

        queueConfig.EvtIoDeviceControl = EvtIoDeviceControl;

        status = WdfIoQueueCreate(
            device,
            &queueConfig,
            WDF_NO_OBJECT_ATTRIBUTES,
            &queue
        );

        if (!NT_SUCCESS(status))
        {
            return status;
        }

        deviceCtx->IoQueue = queue;
    }

    // Create symbolic link
    {
        UNICODE_STRING symLink;
        RtlInitUnicodeString(&symLink, ROTASCOPE_SYMLINK);

        status = WdfDeviceCreateSymbolicLink(device, &symLink);

        if (!NT_SUCCESS(status))
        {
            return status;
        }
    }

    DbgPrint("RotaScope: Device added OK\n");

    return STATUS_SUCCESS;
}

VOID
EvtDeviceContextCleanup(
    _In_ WDFOBJECT Object
)
{
    DEVICE_CONTEXT* deviceCtx;

    deviceCtx = GetDeviceContext((WDFDEVICE)Object);

    if (deviceCtx != NULL)
    {
        FrameCleanup(deviceCtx);

        if (g_GlobalDeviceContext == deviceCtx)
        {
            g_GlobalDeviceContext = NULL;
        }
    }
}
