#include "Shared.h"
#include "Driver.h"
#include <iddcx.h>

extern SharedFrame* g_SharedFrame;

void EvtAssignSwapChain(
    IDDCX_SWAPCHAIN SwapChain,
    const IDDCX_SWAPCHAIN_INFO* Info
) {
    UNREFERENCED_PARAMETER(SwapChain);
    UNREFERENCED_PARAMETER(Info);

    DbgPrint("[RotaScope] Swap chain assigned\n");

    // Initialize the swap chain for frame presentation
}

void EvtReleaseSwapChain(IDDCX_SWAPCHAIN SwapChain)
{
    UNREFERENCED_PARAMETER(SwapChain);
    DbgPrint("[RotaScope] Swap chain released\n");
}

void EvtPresent(IDDCX_SWAPCHAIN SwapChain)
{
    UNREFERENCED_PARAMETER(SwapChain);
    // Present notification - frame should be ready
}

void PresentFrame(IDDCX_SWAPCHAIN hSwapChain)
{
    if (!g_SharedFrame) {
        DbgPrint("[RotaScope] No shared frame buffer available\n");
        return;
    }

    IDDCX_SWAPCHAIN_BUFFER buffer = {};
    NTSTATUS status = IddCxSwapChainGetBuffer(hSwapChain, &buffer);

    if (!NT_SUCCESS(status)) {
        DbgPrint("[RotaScope] Failed to get swap chain buffer: 0x%X\n", status);
        return;
    }

    // Copy frame data from shared memory to swap chain buffer
    BYTE* dst = (BYTE*)buffer.pSurface;
    if (dst) {
        RtlCopyMemory(
            dst,
            g_SharedFrame->pixels,
            FRAME_WIDTH * FRAME_HEIGHT * BYTES_PER_PIXEL
        );
    }

    // Release buffer and present
    IddCxSwapChainReleaseBuffer(hSwapChain, &buffer);
    IddCxSwapChainPresent(hSwapChain);
}
