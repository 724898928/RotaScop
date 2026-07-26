#include "Driver.h"
#include "Shared.h"

extern SharedFrame* g_SharedFrame;

extern "C"
void SubmitFrame(PVOID buffer, SIZE_T size)
{
    if (!g_SharedFrame || !buffer) {
        DbgPrint("[RotaScope] SubmitFrame: invalid parameters\n");
        return;
    }

    SIZE_T frameSize = FRAME_WIDTH * FRAME_HEIGHT * BYTES_PER_PIXEL;
    SIZE_T copySize = (size < frameSize) ? size : frameSize;

    // Copy frame data into shared memory
    RtlCopyMemory(g_SharedFrame->pixels, buffer, copySize);
    InterlockedIncrement(&g_SharedFrame->frameIndex);

    DbgPrint("[RotaScope] Frame submitted: %llu bytes (frame %ld)\n", copySize, g_SharedFrame->frameIndex);

    // TODO:
    // 1. Signal user-mode service that a new frame is available
    // 2. User-mode service reads from shared memory
    // 3. Encodes with NVENC or OpenH264
    // 4. Sends over USB to Android device

    // For MVP, this copies to the swap chain directly
    // In production, this would trigger the encoding + USB pipeline
}
