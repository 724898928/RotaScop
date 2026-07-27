#include "Driver.h"

typedef struct _MONITOR_MODE {
    UINT32 Width;
    UINT32 Height;
    UINT32 VSync;
} MONITOR_MODE;

static const MONITOR_MODE SupportedModes[] =
{
    { 1920, 1080, 60 },
    { 1366,  768, 60 },
    { 1280,  720, 60 },
    { 1024,  768, 60 },
    {  800,  600, 60 },
};

NTSTATUS
EvtAdapterInitFinished(
    IDDCX_ADAPTER AdapterObject,
    const IDARG_OUT_ADAPTER_INIT* Out
)
{
    DEVICE_CONTEXT* deviceCtx;
    IDARG_IN_MONITORCREATE monitorCreate = { 0 };
    IDDCX_MONITOR monitorHandle;
    NTSTATUS status;

    UNREFERENCED_PARAMETER(Out);

    deviceCtx = GetGlobalDeviceContext();

    if (deviceCtx == NULL)
    {
        return STATUS_UNSUCCESSFUL;
    }

    {
        IDDCX_MONITOR_INFO monitorInfo = { 0 };
        IDDCX_TARGET_INFO targetInfo = { 0 };
        IDDCX_MONITOR_MODE modes[ARRAYSIZE(SupportedModes)];
        IDDCX_MONITOR_DESCRIPTION monitorDesc = { 0 };
        GUID containerId = { 0x12345678, 0x1234, 0x1234, { 0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0 } };

        monitorInfo.Size = sizeof(monitorInfo);
        monitorInfo.MonitorType = DISPLAYCONFIG_OUTPUT_TECHNOLOGY_HDMI;
        monitorInfo.MonitorContainerId = containerId;

        targetInfo.Size = sizeof(targetInfo);
        targetInfo.TargetId = 0;
        targetInfo.TargetType = DISPLAYCONFIG_OUTPUT_TECHNOLOGY_HDMI;

        for (UINT32 i = 0; i < ARRAYSIZE(SupportedModes); i++)
        {
            modes[i].Size = sizeof(modes[i]);
            modes[i].VSync = SupportedModes[i].VSync;
            modes[i].HResolution = SupportedModes[i].Width;
            modes[i].VResolution = SupportedModes[i].Height;
            modes[i].VSyncDivisor = 1;
            modes[i].PixelFormat = IDDCX_PIXEL_FORMAT_B8G8R8A8;
        }

        monitorDesc.Size = sizeof(monitorDesc);
        monitorDesc.Type = IDDCX_MONITOR_DESCRIPTION_TYPE_MONITOR_MODE_LIST;
        monitorDesc.MonitorModeList.MonitorModeBuffer = modes;
        monitorDesc.MonitorModeList.MonitorModeCount = ARRAYSIZE(SupportedModes);

        monitorCreate.Size = sizeof(monitorCreate);
        monitorCreate.MonitorInfo = &monitorInfo;
        monitorCreate.TargetInfo = &targetInfo;
        monitorCreate.MonitorDescription = &monitorDesc;
        monitorCreate.pEvtIddCxMonitorAssignSwapChain = EvtMonitorAssignSwapChain;
        monitorCreate.pEvtIddCxMonitorUnassignSwapChain = EvtMonitorUnassignSwapChain;
        monitorCreate.pEvtIddCxParseMonitorDescription = EvtParseMonitorDescription;
        monitorCreate.pEvtIddCxMonitorGetDefaultDescriptionMode = EvtMonitorGetDefaultDescriptionMode;

        status = IddCxMonitorCreate(AdapterObject, &monitorCreate, &monitorHandle);

        if (!NT_SUCCESS(status))
        {
            DbgPrint("RotaScope: IddCxMonitorCreate failed 0x%X\n", status);
            return status;
        }

        deviceCtx->MonitorHandle = monitorHandle;
    }

    {
        IDARG_OUT_MONITORARRIVAL arrival = { 0 };

        status = IddCxMonitorArrival(monitorHandle, &arrival);

        if (!NT_SUCCESS(status))
        {
            DbgPrint("RotaScope: IddCxMonitorArrival failed 0x%X\n", status);
            return status;
        }
    }

    DbgPrint("RotaScope: Monitor created and arrived\n");

    return STATUS_SUCCESS;
}

NTSTATUS
EvtParseMonitorDescription(
    const IDARG_IN_PARSEMONITORDESCRIPTION* pInArgs,
    IDARG_OUT_PARSEMONITORDESCRIPTION* pOutArgs
)
{
    if (pInArgs->MonitorDescription->Type == IDDCX_MONITOR_DESCRIPTION_TYPE_MONITOR_MODE_LIST)
    {
        pOutArgs->MonitorModeBufferOutputCount =
            pInArgs->MonitorDescription->MonitorModeList.MonitorModeCount;
    }

    return STATUS_SUCCESS;
}

NTSTATUS
EvtMonitorGetDefaultDescriptionMode(
    IDDCX_MONITOR MonitorObject,
    IDARG_OUT_MONITOR_GET_DEFAULT_DESCRIPTION_MONITOR_MODE* pOutArgs
)
{
    UNREFERENCED_PARAMETER(MonitorObject);

    pOutArgs->DefaultMonitorMode.Size = sizeof(pOutArgs->DefaultMonitorMode);
    pOutArgs->DefaultMonitorMode.VSync = SupportedModes[0].VSync;
    pOutArgs->DefaultMonitorMode.HResolution = SupportedModes[0].Width;
    pOutArgs->DefaultMonitorMode.VResolution = SupportedModes[0].Height;
    pOutArgs->DefaultMonitorMode.VSyncDivisor = 1;
    pOutArgs->DefaultMonitorMode.PixelFormat = IDDCX_PIXEL_FORMAT_B8G8R8A8;

    return STATUS_SUCCESS;
}
