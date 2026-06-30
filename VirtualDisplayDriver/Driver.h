#pragma once
#include <ntddk.h>

extern "C" NTSTATUS InitIDD(PDRIVER_OBJECT DriverObject);
extern "C" VOID SubmitFrame(PVOID buffer, SIZE_T size);