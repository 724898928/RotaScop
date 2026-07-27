#include "Driver.h"

//
// IddCxStubs.cpp - Stub implementations of IddCx API functions
//
// The real IddCx functions are user-mode only (UMDF) and cannot be called
// from a kernel-mode driver. These stubs allow the driver to compile and link.
// In a production build, the actual IddCx calls would be routed through a
// user-mode companion process (RotaScopeCompanion.exe).
//

NTSTATUS
IddCxDeviceInitialize(
    _In_ WDFDEVICE Device
)
{
    UNREFERENCED_PARAMETER(Device);
    DbgPrint("RotaScope: IddCxDeviceInitialize (stub)\n");
    return STATUS_SUCCESS;
}

NTSTATUS
IddCxAdapterInitAsync(
    _In_ CONST IDARG_IN_ADAPTER_INIT* pInArgs,
    _Out_ IDARG_OUT_ADAPTER_INIT* pOutArgs
)
{
    UNREFERENCED_PARAMETER(pInArgs);

    pOutArgs->AdapterObject = NULL;

    DbgPrint("RotaScope: IddCxAdapterInitAsync (stub)\n");
    return STATUS_SUCCESS;
}

NTSTATUS
IddCxMonitorCreate(
    _In_ IDDCX_ADAPTER hAdapter,
    _In_ CONST IDARG_IN_MONITORCREATE* pInArgs,
    _Out_ IDARG_OUT_MONITORCREATE* pOutArgs
)
{
    UNREFERENCED_PARAMETER(hAdapter);
    UNREFERENCED_PARAMETER(pInArgs);

    pOutArgs->MonitorObject = NULL;

    DbgPrint("RotaScope: IddCxMonitorCreate (stub)\n");
    return STATUS_SUCCESS;
}

NTSTATUS
IddCxMonitorArrival(
    _In_ IDDCX_MONITOR hMonitor,
    _Inout_ IDARG_OUT_MONITORARRIVAL* pOutArgs
)
{
    UNREFERENCED_PARAMETER(hMonitor);

    RtlZeroMemory(pOutArgs, sizeof(*pOutArgs));

    DbgPrint("RotaScope: IddCxMonitorArrival (stub)\n");
    return STATUS_SUCCESS;
}

HRESULT
IddCxSwapChainSetDevice(
    _In_ IDDCX_SWAPCHAIN SwapChainObject,
    _In_ CONST IDARG_IN_SWAPCHAINSETDEVICE* pInArgs
)
{
    UNREFERENCED_PARAMETER(SwapChainObject);
    UNREFERENCED_PARAMETER(pInArgs);

    DbgPrint("RotaScope: IddCxSwapChainSetDevice (stub)\n");
    return S_OK;
}

HRESULT
IddCxSwapChainReleaseAndAcquireBuffer(
    _In_ IDDCX_SWAPCHAIN SwapChainObject,
    _Out_ IDARG_OUT_RELEASEANDACQUIREBUFFER* pOutArgs
)
{
    UNREFERENCED_PARAMETER(SwapChainObject);

    RtlZeroMemory(pOutArgs, sizeof(*pOutArgs));

    DbgPrint("RotaScope: IddCxSwapChainReleaseAndAcquireBuffer (stub)\n");
    return E_PENDING;
}

HRESULT
IddCxSwapChainFinishedProcessingFrame(
    _In_ IDDCX_SWAPCHAIN SwapChainObject
)
{
    UNREFERENCED_PARAMETER(SwapChainObject);

    DbgPrint("RotaScope: IddCxSwapChainFinishedProcessingFrame (stub)\n");
    return S_OK;
}
