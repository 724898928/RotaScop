#include "Driver.h"

extern "C"
NTSTATUS CreateVirtualAdapter()
{
    DbgPrint("Creating Virtual GPU Adapter\n");

    // Windows 会认为：
    // → 有一块 GPU

    return STATUS_SUCCESS;
}