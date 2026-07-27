#pragma once

// ---- Kernel-mode headers first (define _AMD64_ via winnt.h) ----
#include <ntddk.h>
#include <wdm.h>
#include <wdf.h>
#include <dispmprt.h>

// ---- Now block windows.h (kernel types already loaded) ----
#ifndef _WINDOWS_
#define _WINDOWS_
#endif
#ifndef _INC_WINDOWS
#define _INC_WINDOWS
#endif
#define COM_NO_WINDOWS_H

// ---- IddCx types (local shim, no user-mode headers) ----
#include "IddCxShim.h"
#include "Shared.h"

// ---- Device Context ----

typedef struct _DEVICE_CONTEXT {
    WDFDEVICE                   WdfDevice;
    IDDCX_ADAPTER               AdapterHandle;
    IDDCX_MONITOR               MonitorHandle;
    IDDCX_SWAPCHAIN             SwapChain;
    HANDLE                      NextSurfaceAvailable;
    WDFQUEUE                    IoQueue;
    SHARED_FRAME*               SharedFrame;
    HANDLE                      FrameEventHandle;
    PKEVENT                     FrameEventObject;
    KSPIN_LOCK                  FrameLock;
} DEVICE_CONTEXT;

WDF_DECLARE_CONTEXT_TYPE_WITH_NAME(DEVICE_CONTEXT, GetDeviceContext)

// ---- Driver Entry ----

// DriverEntry is declared in Driver.cpp with extern "C" linkage

EVT_WDF_DRIVER_DEVICE_ADD EvtDriverDeviceAdd;
EVT_WDF_DEVICE_CONTEXT_CLEANUP EvtDeviceContextCleanup;

// ---- IddCx Callbacks (registered via IDD_CX_CLIENT_CONFIG) ----

EVT_IDD_CX_ADAPTER_INIT_FINISHED EvtAdapterInitFinished;

EVT_IDD_CX_PARSE_MONITOR_DESCRIPTION EvtParseMonitorDescription;

EVT_IDD_CX_MONITOR_GET_DEFAULT_DESCRIPTION_MODES
EvtMonitorGetDefaultDescriptionModes;

EVT_IDD_CX_MONITOR_ASSIGN_SWAPCHAIN EvtMonitorAssignSwapChain;

EVT_IDD_CX_MONITOR_UNASSIGN_SWAPCHAIN EvtMonitorUnassignSwapChain;

EVT_WDF_IO_QUEUE_IO_DEVICE_CONTROL EvtIoDeviceControl;

// ---- Frame Sharing ----

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
