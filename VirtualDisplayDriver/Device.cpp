#include "Driver.h"

VOID
EvtIoDeviceControl(
    _In_ WDFQUEUE Queue,
    _In_ WDFREQUEST Request,
    _In_ size_t OutputBufferLength,
    _In_ size_t InputBufferLength,
    _In_ ULONG IoControlCode
)
{
    DEVICE_CONTEXT* deviceCtx;
    NTSTATUS status = STATUS_SUCCESS;

    deviceCtx = GetDeviceContext(WdfIoQueueGetDevice(Queue));

    UNREFERENCED_PARAMETER(OutputBufferLength);
    UNREFERENCED_PARAMETER(InputBufferLength);

    switch (IoControlCode)
    {
        case IOCTL_ROTASCOPE_GET_FRAME:
        {
            SHARED_FRAME* frameBuffer;
            size_t bufSize;

            status = WdfRequestRetrieveOutputBuffer(Request, sizeof(SHARED_FRAME), &frameBuffer, &bufSize);

            if (NT_SUCCESS(status) && bufSize >= sizeof(SHARED_FRAME))
            {
                KIRQL oldIrql;

                KeAcquireSpinLock(&deviceCtx->FrameLock, &oldIrql);

                if (deviceCtx->SharedFrame != NULL)
                {
                    RtlCopyMemory(frameBuffer, deviceCtx->SharedFrame, sizeof(SHARED_FRAME));
                }

                KeReleaseSpinLock(&deviceCtx->FrameLock, oldIrql);
            }

            break;
        }

        case IOCTL_ROTASCOPE_WAIT_FRAME:
        {
            if (deviceCtx->FrameEventObject != NULL)
            {
                KeWaitForSingleObject(
                    deviceCtx->FrameEventObject,
                    Executive,
                    KernelMode,
                    FALSE,
                    NULL
                );
            }

            break;
        }

        case IOCTL_ROTASCOPE_SET_RESOLUTION:
        {
            break;
        }

        default:
            status = STATUS_INVALID_DEVICE_REQUEST;
            break;
    }

    WdfRequestComplete(Request, status);
}

VOID
SetSharedFrame(
    _In_ DEVICE_CONTEXT* DeviceCtx,
    _In_reads_bytes_(BufferSize) const BYTE* Buffer,
    _In_ ULONG Width,
    _In_ ULONG Height,
    _In_ ULONG Stride
)
{
    SHARED_FRAME* frame;
    ULONG copyBytes;

    KeAcquireSpinLockAtDpcLevel(&DeviceCtx->FrameLock);

    if (DeviceCtx->SharedFrame == NULL)
    {
        DeviceCtx->SharedFrame = (SHARED_FRAME*)
            ExAllocatePool2(POOL_FLAG_NON_PAGED, sizeof(SHARED_FRAME), 'FRSR');
    }

    frame = DeviceCtx->SharedFrame;

    if (frame != NULL)
    {
        copyBytes = (Stride * Height < SHARED_FRAME_BUFFER_SIZE) ?
                     Stride * Height : SHARED_FRAME_BUFFER_SIZE;

        RtlCopyMemory(frame->Buffer, Buffer, copyBytes);

        frame->Width = Width;
        frame->Height = Height;
        frame->Stride = Stride;
        InterlockedExchange(&frame->FrameReady, 1);

        if (DeviceCtx->FrameEventObject != NULL)
        {
            KeSetEvent(DeviceCtx->FrameEventObject, IO_NO_INCREMENT, FALSE);
        }
    }

    KeReleaseSpinLockFromDpcLevel(&DeviceCtx->FrameLock);
}
