#include "Driver.h"

static DEVICE_CONTEXT* g_GlobalDeviceContext = NULL;

DEVICE_CONTEXT*
GetGlobalDeviceContext(VOID)
{
    return g_GlobalDeviceContext;
}

NTSTATUS
DriverEntry(
    PDRIVER_OBJECT  DriverObject,
    PUNICODE_STRING RegistryPath
)
{
    WDF_DRIVER_CONFIG config;
    IDD_CX_INIT_CONFIG iddConfig;
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

    IDD_CX_INIT_CONFIG_INIT(&iddConfig);

    status = IddCxInitialize(DriverObject, &iddConfig);

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

    {
        IDDCX_ADAPTER adapter = { 0 };
        IDARG_IN_ADAPTER_INIT adapterInit = { 0 };

        adapter.Size = sizeof(adapter);

        adapterInit.Adapter = &adapter;
        adapterInit.pEvtIddCxAdapterInitFinished = EvtAdapterInitFinished;

        status = IddCxAdapterCreate(device, &adapterInit, &deviceCtx->AdapterHandle);

        if (!NT_SUCCESS(status))
        {
            DbgPrint("RotaScope: IddCxAdapterCreate failed 0x%X\n", status);
            return status;
        }
    }

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

EVT_WDF_DEVICE_CONTEXT_CLEANUP EvtDeviceContextCleanup;

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
