#include "Driver.h"

extern "C"
NTSTATUS CreateVirtualMonitor()
{
    DbgPrint("Creating Virtual Monitor\n");

    // 告诉 Windows：
    // “插入了一块显示器”

    return STATUS_SUCCESS;
}