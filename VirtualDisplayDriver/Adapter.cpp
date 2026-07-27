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

static VOID
FillMonitorMode(
    _Out_ IDDCX_MONITOR_MODE* Mode,
    _In_ UINT32 Width,
    _In_ UINT32 Height,
    _In_ UINT32 VSyncHz
)
{
    RtlZeroMemory(Mode, sizeof(*Mode));
    Mode->Size = sizeof(*Mode);
    Mode->Origin = IDDCX_MONITOR_MODE_ORIGIN_DRIVER;

    Mode->MonitorVideoSignalInfo.pixelRate = (UINT64)Width * Height * VSyncHz;
    Mode->MonitorVideoSignalInfo.hSyncFreq.Numerator = VSyncHz;
    Mode->MonitorVideoSignalInfo.hSyncFreq.Denominator = 1;
    Mode->MonitorVideoSignalInfo.vSyncFreq.Numerator = VSyncHz;
    Mode->MonitorVideoSignalInfo.vSyncFreq.Denominator = 1;
    Mode->MonitorVideoSignalInfo.activeSize.cx = Width;
    Mode->MonitorVideoSignalInfo.activeSize.cy = Height;
    Mode->MonitorVideoSignalInfo.totalSize.cx = Width;
    Mode->MonitorVideoSignalInfo.totalSize.cy = Height;
    Mode->MonitorVideoSignalInfo.scanLineOrdering = DISPLAYCONFIG_SCANLINE_ORDERING_PROGRESSIVE;
}

NTSTATUS
EvtAdapterInitFinished(
    _In_ IDDCX_ADAPTER AdapterObject,
    _In_ const IDARG_IN_ADAPTER_INIT_FINISHED* pInArgs
)
{
    DEVICE_CONTEXT* deviceCtx;
    IDARG_IN_MONITORCREATE monitorCreate = { 0 };
    IDARG_OUT_MONITORCREATE monitorOut = { 0 };
    NTSTATUS status;

    UNREFERENCED_PARAMETER(pInArgs);

    deviceCtx = GetGlobalDeviceContext();

    if (deviceCtx == NULL)
    {
        return STATUS_UNSUCCESSFUL;
    }

    {
        IDDCX_MONITOR_INFO monitorInfo = { 0 };
        IDDCX_MONITOR_DESCRIPTION monitorDesc = { 0 };
        GUID containerId = { 0x12345678, 0x1234, 0x1234, { 0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0 } };

        monitorInfo.Size = sizeof(monitorInfo);
        monitorInfo.MonitorType = DISPLAYCONFIG_OUTPUT_TECHNOLOGY_HDMI;
        monitorInfo.ConnectorIndex = 0;
        monitorInfo.MonitorContainerId = containerId;

        // No EDID — provide empty monitor description
        monitorDesc.Size = sizeof(monitorDesc);
        monitorDesc.Type = IDDCX_MONITOR_DESCRIPTION_TYPE_EDID;
        monitorDesc.DataSize = 0;
        monitorDesc.pData = NULL;

        monitorInfo.MonitorDescription = monitorDesc;

        monitorCreate.pMonitorInfo = &monitorInfo;

        status = IddCxMonitorCreate(AdapterObject, &monitorCreate, &monitorOut);

        if (!NT_SUCCESS(status))
        {
            DbgPrint("RotaScope: IddCxMonitorCreate failed 0x%X\n", status);
            return status;
        }

        deviceCtx->MonitorHandle = monitorOut.MonitorObject;
    }

    {
        IDARG_OUT_MONITORARRIVAL arrival = { 0 };

        status = IddCxMonitorArrival(deviceCtx->MonitorHandle, &arrival);

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
    _In_ const IDARG_IN_PARSEMONITORDESCRIPTION* pInArgs,
    _Out_ IDARG_OUT_PARSEMONITORDESCRIPTION* pOutArgs
)
{
    pOutArgs->PreferredMonitorModeIdx = NO_PREFERRED_MODE;

    if (pInArgs->MonitorDescription.Type == IDDCX_MONITOR_DESCRIPTION_TYPE_EDID &&
        pInArgs->MonitorDescription.DataSize == 0)
    {
        // No EDID: fill the caller's buffer with our supported modes
        UINT32 count = ARRAYSIZE(SupportedModes);

        if (pInArgs->pMonitorModes != NULL && pInArgs->MonitorModeBufferInputCount >= count)
        {
            for (UINT32 i = 0; i < count; i++)
            {
                FillMonitorMode(
                    &pInArgs->pMonitorModes[i],
                    SupportedModes[i].Width,
                    SupportedModes[i].Height,
                    SupportedModes[i].VSync
                );
            }
            pOutArgs->PreferredMonitorModeIdx = 0;
        }

        pOutArgs->MonitorModeBufferOutputCount = count;
    }
    else
    {
        pOutArgs->MonitorModeBufferOutputCount = 0;
    }

    return STATUS_SUCCESS;
}

NTSTATUS
EvtMonitorGetDefaultDescriptionModes(
    _In_ IDDCX_MONITOR MonitorObject,
    _In_ const IDARG_IN_GETDEFAULTDESCRIPTIONMODES* pInArgs,
    _Out_ IDARG_OUT_GETDEFAULTDESCRIPTIONMODES* pOutArgs
)
{
    UINT32 count = ARRAYSIZE(SupportedModes);

    UNREFERENCED_PARAMETER(MonitorObject);

    if (pInArgs->pDefaultMonitorModes != NULL &&
        pInArgs->DefaultMonitorModeBufferInputCount >= count)
    {
        for (UINT32 i = 0; i < count; i++)
        {
            FillMonitorMode(
                &pInArgs->pDefaultMonitorModes[i],
                SupportedModes[i].Width,
                SupportedModes[i].Height,
                SupportedModes[i].VSync
            );
        }
        pOutArgs->PreferredMonitorModeIdx = 0;
    }

    pOutArgs->DefaultMonitorModeBufferOutputCount = count;

    return STATUS_SUCCESS;
}
