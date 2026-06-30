#include "Driver.h"
#include <iddcx.h>


IDDCX_MONITOR_MODE g_Modes[] = {
{ {1920, 1080}, 120 },
{ {1280, 720}, 120 }
};


void CreateMonitor(IDDCX_ADAPTER Adapter)
{
IDDCX_MONITOR_INFO info = {};
info.Size = sizeof(info);
info.MonitorDescription.Type = IDDCX_MONITOR_DESCRIPTION_TYPE_GENERIC;


IddCxMonitorCreate(Adapter, &info, nullptr);
}