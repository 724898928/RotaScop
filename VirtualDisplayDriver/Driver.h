#pragma once
#include <ntddk.h>
#include <dispmprt.h>
#include <iddcx.h>

// Driver entry and unload
extern "C" NTSTATUS DriverEntry(PDRIVER_OBJECT DriverObject, PUNICODE_STRING RegistryPath);
extern "C" VOID DriverUnload(PDRIVER_OBJECT DriverObject);

// IDD initialization
extern "C" NTSTATUS InitIDD(PDRIVER_OBJECT DriverObject);

// Frame submission from user-mode
extern "C" VOID SubmitFrame(PVOID buffer, SIZE_T size);

// Adapter creation
NTSTATUS CreateVirtualAdapter(PDRIVER_OBJECT DriverObject);

// Monitor and device callbacks
void CreateMonitor(IDDCX_ADAPTER Adapter);
NTSTATUS IddDeviceIoControl(PVOID Context, PVOID InputBuffer, ULONG InputBufferLength,
                             PVOID OutputBuffer, ULONG OutputBufferLength, PULONG ReturnedData);

// Swap chain callbacks
void EvtAssignSwapChain(IDDCX_SWAPCHAIN SwapChain, const IDDCX_SWAPCHAIN_INFO* Info);
void EvtReleaseSwapChain(IDDCX_SWAPCHAIN SwapChain);
void EvtPresent(IDDCX_SWAPCHAIN SwapChain);

// Global shared frame buffer
extern struct SharedFrame* g_SharedFrame;
