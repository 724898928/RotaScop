#include "Driver.h"

NTSTATUS
EvtMonitorAssignSwapChain(
    _In_ IDDCX_MONITOR MonitorObject,
    _In_ const IDARG_IN_SETSWAPCHAIN* pInArgs
)
{
    DEVICE_CONTEXT* deviceCtx;
    IDARG_IN_SWAPCHAINSETDEVICE setDevice = { 0 };
    HRESULT hr;

    UNREFERENCED_PARAMETER(MonitorObject);

    deviceCtx = GetGlobalDeviceContext();

    if (deviceCtx == NULL)
    {
        return STATUS_UNSUCCESSFUL;
    }

    deviceCtx->SwapChain = pInArgs->hSwapChain;
    deviceCtx->NextSurfaceAvailable = pInArgs->hNextSurfaceAvailable;

    // Note: IDARG_IN_SWAPCHAINSETDEVICE.pDevice would normally point to the
    // IDXGIDevice. For our stub path, we pass NULL.
    setDevice.pDevice = NULL;

    hr = IddCxSwapChainSetDevice(
        deviceCtx->SwapChain,
        &setDevice
    );

    if (FAILED(hr))
    {
        deviceCtx->SwapChain = NULL;
        deviceCtx->NextSurfaceAvailable = NULL;
        return STATUS_UNSUCCESSFUL;
    }

    DbgPrint("RotaScope: SwapChain assigned\n");

    return STATUS_SUCCESS;
}

NTSTATUS
EvtMonitorUnassignSwapChain(
    _In_ IDDCX_MONITOR MonitorObject
)
{
    DEVICE_CONTEXT* deviceCtx;

    UNREFERENCED_PARAMETER(MonitorObject);

    deviceCtx = GetGlobalDeviceContext();

    if (deviceCtx != NULL)
    {
        deviceCtx->SwapChain = NULL;
        deviceCtx->NextSurfaceAvailable = NULL;
    }

    DbgPrint("RotaScope: SwapChain unassigned\n");

    return STATUS_SUCCESS;
}
