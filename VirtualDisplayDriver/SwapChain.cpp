#include "Shared.h"
#include <iddcx.h>
#include "Driver.h"
#include <iddcx.h>


void EvtAssignSwapChain(
IDDCX_SWAPCHAIN SwapChain,
const IDDCX_SWAPCHAIN_INFO* Info
) {
// TODO: 写入共享内存帧
}


void EvtReleaseSwapChain(IDDCX_SWAPCHAIN SwapChain) {}


void EvtPresent(IDDCX_SWAPCHAIN SwapChain) {
// Present 通知
}

extern SharedFrame* g_SharedFrame;


void PresentFrame(IDDCX_SWAPCHAIN hSwapChain)
{
IDDCX_SWAPCHAIN_BUFFER buffer;
if (IddCxSwapChainGetBuffer(hSwapChain, &buffer) != STATUS_SUCCESS)
return;


BYTE* dst = (BYTE*)buffer.pSurface;
memcpy(dst, g_SharedFrame->pixels,
FRAME_WIDTH * FRAME_HEIGHT * BYTES_PER_PIXEL);


IddCxSwapChainReleaseBuffer(hSwapChain, &buffer);
IddCxSwapChainPresent(hSwapChain);
}