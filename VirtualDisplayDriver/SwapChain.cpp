#include "Driver.h"

NTSTATUS
EvtSwapChainSetDevice(
    IDDCX_SWAPCHAIN SwapChainObject,
    const IDARG_IN_SETDEVICE* pInArgs
)
{
    UNREFERENCED_PARAMETER(SwapChainObject);
    UNREFERENCED_PARAMETER(pInArgs);

    return STATUS_SUCCESS;
}

NTSTATUS
EvtSwapChainSetSwapChain(
    IDDCX_SWAPCHAIN SwapChainObject,
    const IDARG_IN_SETSWAPCHAIN* pInArgs
)
{
    DEVICE_CONTEXT* deviceCtx;

    deviceCtx = GetGlobalDeviceContext();

    if (deviceCtx != NULL)
    {
        deviceCtx->SwapChain = pInArgs->SwapChain;
    }

    UNREFERENCED_PARAMETER(SwapChainObject);

    return STATUS_SUCCESS;
}

NTSTATUS
EvtSwapChainReleaseSwapChain(
    IDDCX_SWAPCHAIN SwapChainObject
)
{
    DEVICE_CONTEXT* deviceCtx;

    deviceCtx = GetGlobalDeviceContext();

    if (deviceCtx != NULL)
    {
        deviceCtx->SwapChain = NULL;
    }

    UNREFERENCED_PARAMETER(SwapChainObject);

    return STATUS_SUCCESS;
}

NTSTATUS
EvtSwapChainProcessFrame(
    IDDCX_SWAPCHAIN SwapChainObject,
    const IDARG_IN_PROCESSFRAME* pInArgs
)
{
    DEVICE_CONTEXT* deviceCtx;
    IDARG_IN_RELEASEANDACQUIREBUFFER acquireIn = { 0 };
    IDARG_OUT_RELEASEANDACQUIREBUFFER acquireOut = { 0 };
    NTSTATUS status;

    deviceCtx = GetGlobalDeviceContext();

    if (deviceCtx == NULL)
    {
        return STATUS_UNSUCCESSFUL;
    }

    UNREFERENCED_PARAMETER(SwapChainObject);
    UNREFERENCED_PARAMETER(pInArgs);

    acquireIn.Size = sizeof(acquireIn);

    status = IddCxSwapChainReleaseAndAcquireBuffer(
        deviceCtx->SwapChain,
        &acquireIn,
        &acquireOut
    );

    if (!NT_SUCCESS(status))
    {
        return status;
    }

    {
        IDDCX_SWAPCHAIN_BUFFER_INFO bufferInfo = { 0 };
        SIZE_T infoSize;

        bufferInfo.Size = sizeof(bufferInfo);

        status = IddCxSwapChainGetBufferInfo(
            deviceCtx->SwapChain,
            acquireOut.BufferIndex,
            &bufferInfo,
            sizeof(bufferInfo),
            &infoSize
        );

        if (NT_SUCCESS(status) &&
            bufferInfo.pSurface != NULL &&
            bufferInfo.FrameBufferSize > 0)
        {
            SetSharedFrame(
                deviceCtx,
                (const BYTE*)bufferInfo.pSurface,
                bufferInfo.FrameBufferWidth,
                bufferInfo.FrameBufferHeight,
                bufferInfo.FrameBufferStride
            );
        }
    }

    status = IddCxSwapChainFinishedProcessingFrame(
        deviceCtx->SwapChain
    );

    return status;
}
