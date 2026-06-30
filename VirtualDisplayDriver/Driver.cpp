#include <ntddk.h>
#include "Driver.h"

extern "C"
NTSTATUS DriverEntry(PDRIVER_OBJECT DriverObject, PUNICODE_STRING RegistryPath)
{
    UNREFERENCED_PARAMETER(RegistryPath);

    DbgPrint("Virtual Display Driver Loaded\n");

    DriverObject->DriverUnload = DriverUnload;

    return InitIDD(DriverObject);
}

extern "C"
VOID DriverUnload(PDRIVER_OBJECT DriverObject)
{
    UNREFERENCED_PARAMETER(DriverObject);
    DbgPrint("Driver Unloaded\n");
}