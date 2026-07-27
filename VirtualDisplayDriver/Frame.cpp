#include "Driver.h"

NTSTATUS
FrameInitialize(
    _In_ DEVICE_CONTEXT* DeviceCtx
)
{
    SHARED_FRAME* frame;
    UNICODE_STRING eventName;
    HANDLE hEvent;
    PKEVENT pEvent;

    frame = (SHARED_FRAME*)ExAllocatePool2(
        POOL_FLAG_NON_PAGED,
        sizeof(SHARED_FRAME),
        'FRSR'
    );

    if (frame == NULL)
    {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    RtlZeroMemory(frame, sizeof(SHARED_FRAME));

    DeviceCtx->SharedFrame = frame;

    RtlInitUnicodeString(&eventName, L"\\BaseNamedObjects\\RotaScopeFrameEvent");

    pEvent = IoCreateSynchronizationEvent(
        &eventName,
        &hEvent
    );

    if (pEvent == NULL)
    {
        ExFreePool(frame);
        DeviceCtx->SharedFrame = NULL;
        return STATUS_UNSUCCESSFUL;
    }

    DeviceCtx->FrameEventObject = pEvent;
    DeviceCtx->FrameEventHandle = hEvent;

    KeClearEvent(pEvent);

    DbgPrint("RotaScope: Frame buffer initialized\n");

    return STATUS_SUCCESS;
}

VOID
FrameCleanup(
    _In_ DEVICE_CONTEXT* DeviceCtx
)
{
    if (DeviceCtx->SharedFrame != NULL)
    {
        ExFreePool(DeviceCtx->SharedFrame);
        DeviceCtx->SharedFrame = NULL;
    }

    if (DeviceCtx->FrameEventHandle != NULL)
    {
        ZwClose(DeviceCtx->FrameEventHandle);
        DeviceCtx->FrameEventHandle = NULL;
    }

    DeviceCtx->FrameEventObject = NULL;

    DbgPrint("RotaScope: Frame buffer cleaned up\n");
}
