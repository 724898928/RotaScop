#pragma once

#include <ntddk.h>
#include <dispmprt.h>
#include <iddcx.h>
#include <wdm.h>
#include <wdf.h>
#include "Shared.h"

typedef struct _DEVICE_CONTEXT {
    WDFDEVICE                   WdfDevice;
    IDDCX_ADAPTER               AdapterHandle;
    IDDCX_MONITOR               MonitorHandle;
    IDDCX_SWAPCHAIN             SwapChain;
    WDFQUEUE                    IoQueue;
    SHARED_FRAME*               SharedFrame;
    HANDLE                      FrameEventHandle;
    PKEVENT                     FrameEventObject;
    KSPIN_LOCK                  FrameLock;
} DEVICE_CONTEXT;

WDF_DECLARE_CONTEXT_TYPE_WITH_NAME(DEVICE_CONTEXT, GetDeviceContext)

DRIVER_INITIALIZE DriverEntry;

NTSTATUS
DriverEntry(
    PDRIVER_OBJECT  DriverObject,
    PUNICODE_STRING RegistryPath
);

EVT_WDF_DRIVER_DEVICE_ADD EvtDriverDeviceAdd;

EVT_IDD_CX_ADAPTER_INIT_FINISHED EvtAdapterInitFinished;

EVT_IDD_CX_PARSE_MONITOR_DESCRIPTION EvtParseMonitorDescription;

EVT_IDD_CX_MONITOR_GET_DEFAULT_DESCRIPTION_MONITOR_MODE
EvtMonitorGetDefaultDescriptionMode;

EVT_IDD_CX_MONITOR_ASSIGN_SWAPCHAIN EvtMonitorAssignSwapChain;

EVT_IDD_CX_MONITOR_UNASSIGN_SWAPCHAIN EvtMonitorUnassignSwapChain;

EVT_IDD_CX_SWAPCHAIN_SET_DEVICE EvtSwapChainSetDevice;

EVT_IDD_CX_SWAPCHAIN_SET_SWAPCHAIN EvtSwapChainSetSwapChain;

EVT_IDD_CX_SWAPCHAIN_RELEASE_SWAPCHAIN EvtSwapChainReleaseSwapChain;

EVT_IDD_CX_SWAPCHAIN_PROCESS_FRAME EvtSwapChainProcessFrame;

EVT_WDF_IO_QUEUE_IO_DEVICE_CONTROL EvtIoDeviceControl;

NTSTATUS
FrameInitialize(
    _In_ DEVICE_CONTEXT* DeviceCtx
);

VOID
FrameCleanup(
    _In_ DEVICE_CONTEXT* DeviceCtx
);

DEVICE_CONTEXT*
GetGlobalDeviceContext(VOID);

VOID
SetSharedFrame(
    _In_ DEVICE_CONTEXT* DeviceCtx,
    _In_reads_bytes_(BufferSize) const BYTE* Buffer,
    _In_ ULONG Width,
    _In_ ULONG Height,
    _In_ ULONG Stride
);
