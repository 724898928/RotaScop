#include "Driver.h"

NTSTATUS
EvtMonitorAssignSwapChain(
    IDDCX_MONITOR MonitorObject,
    const IDARG_IN_SETSWAPCHAIN* pInArgs,
    IDARG_OUT_SETSWAPCHAIN* pOutArgs
)
{
    DEVICE_CONTEXT* deviceCtx;
    IDARG_IN_SWAPCHAINSETDEVICE setDevice = { 0 };
    NTSTATUS status;

    UNREFERENCED_PARAMETER(MonitorObject);

    deviceCtx = GetGlobalDeviceContext();

    if (deviceCtx == NULL)
    {
        return STATUS_UNSUCCESSFUL;
    }

    deviceCtx->SwapChain = pInArgs->SwapChain;

    setDevice.Size = sizeof(setDevice);
    setDevice.pEvtIddCxSwapChainSetDevice = EvtSwapChainSetDevice;
    setDevice.pEvtIddCxSwapChainSetSwapChain = EvtSwapChainSetSwapChain;
    setDevice.pEvtIddCxSwapChainReleaseSwapChain = EvtSwapChainReleaseSwapChain;
    setDevice.pEvtIddCxSwapChainProcessFrame = EvtSwapChainProcessFrame;

    status = IddCxSwapChainSetDevice(
        deviceCtx->SwapChain,
        &setDevice
    );

    if (!NT_SUCCESS(status))
    {
        deviceCtx->SwapChain = NULL;
        return status;
    }

    pOutArgs->Result = IDDCX_SETSWAPCHAIN_RESULT_OK;

    DbgPrint("RotaScope: SwapChain assigned with callbacks\n");

    return STATUS_SUCCESS;
}

NTSTATUS
EvtMonitorUnassignSwapChain(
    IDDCX_MONITOR MonitorObject,
    const IDARG_OUT_RELEASESWAPCHAIN* pOutArgs
)
{
    DEVICE_CONTEXT* deviceCtx;

    UNREFERENCED_PARAMETER(MonitorObject);
    UNREFERENCED_PARAMETER(pOutArgs);

    deviceCtx = GetGlobalDeviceContext();

    if (deviceCtx != NULL)
    {
        deviceCtx->SwapChain = NULL;
    }

    DbgPrint("RotaScope: SwapChain unassigned\n");

    return STATUS_SUCCESS;
}
