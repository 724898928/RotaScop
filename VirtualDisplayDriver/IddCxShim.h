#pragma once
//
// IddCxShim.h - Kernel-mode IddCx type definitions
//
// The real <iddcx.h> (SDK um/iddcx/1.10/) includes user-mode headers
// (Opmapi.h, Dxgi.h, d3d11_4.h, wingdi.h) that cannot compile under /kernel.
// This shim provides the subset of types and function prototypes needed by
// VirtualDisplayDriver without pulling in any user-mode headers.
//
// REQUIREMENT: ntddk.h / wdm.h / wdf.h must be included BEFORE this header.
//

#include <ntddk.h>

// ---- HRESULT / COM macros (from winerror.h, unavailable under /kernel) ----

#ifndef S_OK
#define S_OK ((HRESULT)0L)
#endif
#ifndef S_FALSE
#define S_FALSE ((HRESULT)1L)
#endif
#ifndef E_PENDING
#define E_PENDING ((HRESULT)0x80000005L)
#endif
#ifndef E_FAIL
#define E_FAIL ((HRESULT)0x80004005L)
#endif
#ifndef SUCCEEDED
#define SUCCEEDED(hr) (((HRESULT)(hr)) >= 0)
#endif
#ifndef FAILED
#define FAILED(hr) (((HRESULT)(hr)) < 0)
#endif

// ---- Forward declarations for user-mode COM interfaces (pointer-only) ----

struct IDXGIDevice;
struct IDXGIResource;

// ---- Minimal user-mode type definitions (not available in km/) ----

#ifndef DISPLAYCONFIG_VIDEO_OUTPUT_TECHNOLOGY
typedef enum _DISPLAYCONFIG_VIDEO_OUTPUT_TECHNOLOGY {
    DISPLAYCONFIG_OUTPUT_TECHNOLOGY_OTHER             = -1,
    DISPLAYCONFIG_OUTPUT_TECHNOLOGY_HD15              = 0,
    DISPLAYCONFIG_OUTPUT_TECHNOLOGY_SVIDEO            = 1,
    DISPLAYCONFIG_OUTPUT_TECHNOLOGY_COMPOSITE_VIDEO   = 2,
    DISPLAYCONFIG_OUTPUT_TECHNOLOGY_COMPONENT_VIDEO    = 3,
    DISPLAYCONFIG_OUTPUT_TECHNOLOGY_DVI               = 4,
    DISPLAYCONFIG_OUTPUT_TECHNOLOGY_HDMI              = 5,
    DISPLAYCONFIG_OUTPUT_TECHNOLOGY_LVDS              = 6,
    DISPLAYCONFIG_OUTPUT_TECHNOLOGY_D_JPN             = 8,
    DISPLAYCONFIG_OUTPUT_TECHNOLOGY_SDI               = 9,
    DISPLAYCONFIG_OUTPUT_TECHNOLOGY_DISPLAYPORT_EXTERNAL = 10,
    DISPLAYCONFIG_OUTPUT_TECHNOLOGY_DISPLAYPORT_EMBEDDED = 11,
    DISPLAYCONFIG_OUTPUT_TECHNOLOGY_UDI_EXTERNAL      = 12,
    DISPLAYCONFIG_OUTPUT_TECHNOLOGY_UDI_EMBEDDED      = 13,
    DISPLAYCONFIG_OUTPUT_TECHNOLOGY_INTERNAL          = 0x80000000,
    DISPLAYCONFIG_OUTPUT_TECHNOLOGY_FORCE_UINT32      = 0xFFFFFFFF
} DISPLAYCONFIG_VIDEO_OUTPUT_TECHNOLOGY;
#endif

#ifndef DISPLAYCONFIG_SCANLINE_ORDERING
typedef enum _DISPLAYCONFIG_SCANLINE_ORDERING {
    DISPLAYCONFIG_SCANLINE_ORDERING_UNSPECIFIED                 = 0,
    DISPLAYCONFIG_SCANLINE_ORDERING_PROGRESSIVE                 = 1,
    DISPLAYCONFIG_SCANLINE_ORDERING_INTERLACED                  = 2,
    DISPLAYCONFIG_SCANLINE_ORDERING_INTERLACED_UPPERFIELDFIRST  = DISPLAYCONFIG_SCANLINE_ORDERING_INTERLACED,
    DISPLAYCONFIG_SCANLINE_ORDERING_INTERLACED_LOWERFIELDFIRST  = 3,
    DISPLAYCONFIG_SCANLINE_ORDERING_FORCE_UINT32                = 0xFFFFFFFF
} DISPLAYCONFIG_SCANLINE_ORDERING;
#endif

#ifndef _DISPLAYCONFIG_RATIONAL
typedef struct _DISPLAYCONFIG_RATIONAL {
    UINT32 Numerator;
    UINT32 Denominator;
} DISPLAYCONFIG_RATIONAL;
#endif

#ifndef _DISPLAYCONFIG_2DREGION
typedef struct _DISPLAYCONFIG_2DREGION {
    UINT32 cx;
    UINT32 cy;
} DISPLAYCONFIG_2DREGION;
#endif

#ifndef _DISPLAYCONFIG_VIDEO_SIGNAL_INFO_DEFINED
#define _DISPLAYCONFIG_VIDEO_SIGNAL_INFO_DEFINED
typedef struct _DISPLAYCONFIG_VIDEO_SIGNAL_INFO {
    UINT64 pixelRate;
    DISPLAYCONFIG_RATIONAL hSyncFreq;
    DISPLAYCONFIG_RATIONAL vSyncFreq;
    DISPLAYCONFIG_2DREGION activeSize;
    DISPLAYCONFIG_2DREGION totalSize;
    union {
        struct {
            UINT32 videoStandard : 16;
            UINT32 vSyncFreqDivider : 6;
            UINT32 reserved : 10;
        } AdditionalSignalInfo;
        UINT32 videoStandard;
    } DUMMYUNIONNAME;
    DISPLAYCONFIG_SCANLINE_ORDERING scanLineOrdering;
} DISPLAYCONFIG_VIDEO_SIGNAL_INFO;
#endif

#ifndef _DISPLAYCONFIG_TARGET_MODE
typedef struct _DISPLAYCONFIG_TARGET_MODE {
    DISPLAYCONFIG_VIDEO_SIGNAL_INFO targetVideoSignalInfo;
} DISPLAYCONFIG_TARGET_MODE;
#endif

// ---- Opaque handle types (matching DECLARE_HANDLE from real IddCx.h) ----

DECLARE_HANDLE(IDDCX_ADAPTER);
DECLARE_HANDLE(IDDCX_MONITOR);
DECLARE_HANDLE(IDDCX_SWAPCHAIN);

// ---- Enum Declarations ----

enum IDDCX_ADAPTER_FLAGS : UINT {
    IDDCX_ADAPTER_FLAGS_NONE                        = 0,
    IDDCX_ADAPTER_FLAGS_USE_SMALLEST_MODE           = 0x1,
    IDDCX_ADAPTER_FLAGS_CAN_USE_MOVE_REGIONS        = 0x2,
    IDDCX_ADAPTER_FLAGS_REMOTE_SESSION_DRIVER       = 0x4,
    IDDCX_ADAPTER_FLAGS_PREFER_PHYSICALLY_CONTIGUOUS = 0x8,
};

enum IDDCX_TRANSMISSION_TYPE : UINT {
    IDDCX_TRANSMISSION_TYPE_UNINITIALIZED    = 0,
    IDDCX_TRANSMISSION_TYPE_WIRED_USB        = 0x1,
    IDDCX_TRANSMISSION_TYPE_OTHER            = 0xFFFFFFFF,
};

enum IDDCX_FEATURE_IMPLEMENTATION : UINT {
    IDDCX_FEATURE_IMPLEMENTATION_UNINITIALIZED = 0,
    IDDCX_FEATURE_IMPLEMENTATION_NONE          = 1,
    IDDCX_FEATURE_IMPLEMENTATION_HARDWARE      = 2,
    IDDCX_FEATURE_IMPLEMENTATION_SOFTWARE      = 3,
};

enum IDDCX_MONITOR_DESCRIPTION_TYPE : UINT {
    IDDCX_MONITOR_DESCRIPTION_TYPE_UNINITIALIZED      = 0,
    IDDCX_MONITOR_DESCRIPTION_TYPE_EDID               = 1,
    IDDCX_MONITOR_DESCRIPTION_TYPE_DISPLAYID_AND_EDID = 2,
};

enum IDDCX_MONITOR_MODE_ORIGIN : UINT {
    IDDCX_MONITOR_MODE_ORIGIN_UNINITIALIZED       = 0,
    IDDCX_MONITOR_MODE_ORIGIN_MONITORDESCRIPTOR   = 1,
    IDDCX_MONITOR_MODE_ORIGIN_DRIVER              = 2,
};

// ---- Callback function pointer typedefs ----

// Forward declare all callback arg structs
struct IDARG_IN_PARSEMONITORDESCRIPTION;
struct IDARG_OUT_PARSEMONITORDESCRIPTION;
struct IDARG_IN_ADAPTER_INIT_FINISHED;
struct IDARG_IN_GETDEFAULTDESCRIPTIONMODES;
struct IDARG_OUT_GETDEFAULTDESCRIPTIONMODES;
struct IDARG_IN_SETSWAPCHAIN;

// EVT_IDD_CX_DEVICE_IO_CONTROL
typedef
_Function_class_(EVT_IDD_CX_DEVICE_IO_CONTROL)
_IRQL_requires_same_
VOID
NTAPI
EVT_IDD_CX_DEVICE_IO_CONTROL(
    _In_ WDFDEVICE Device,
    _In_ WDFREQUEST Request,
    _In_ size_t OutputBufferLength,
    _In_ size_t InputBufferLength,
    _In_ ULONG IoControlCode
);
typedef EVT_IDD_CX_DEVICE_IO_CONTROL *PFN_IDD_CX_DEVICE_IO_CONTROL;

// EVT_IDD_CX_PARSE_MONITOR_DESCRIPTION
typedef
_Function_class_(EVT_IDD_CX_PARSE_MONITOR_DESCRIPTION)
_IRQL_requires_same_
NTSTATUS
NTAPI
EVT_IDD_CX_PARSE_MONITOR_DESCRIPTION(
    _In_ const IDARG_IN_PARSEMONITORDESCRIPTION* pInArgs,
    _Out_ IDARG_OUT_PARSEMONITORDESCRIPTION* pOutArgs
);
typedef EVT_IDD_CX_PARSE_MONITOR_DESCRIPTION *PFN_IDD_CX_PARSE_MONITOR_DESCRIPTION;

// EVT_IDD_CX_ADAPTER_INIT_FINISHED
typedef
_Function_class_(EVT_IDD_CX_ADAPTER_INIT_FINISHED)
_IRQL_requires_same_
NTSTATUS
NTAPI
EVT_IDD_CX_ADAPTER_INIT_FINISHED(
    _In_ IDDCX_ADAPTER AdapterObject,
    _In_ const IDARG_IN_ADAPTER_INIT_FINISHED* pInArgs
);
typedef EVT_IDD_CX_ADAPTER_INIT_FINISHED *PFN_IDD_CX_ADAPTER_INIT_FINISHED;

// EVT_IDD_CX_MONITOR_GET_DEFAULT_DESCRIPTION_MODES
typedef
_Function_class_(EVT_IDD_CX_MONITOR_GET_DEFAULT_DESCRIPTION_MODES)
_IRQL_requires_same_
NTSTATUS
NTAPI
EVT_IDD_CX_MONITOR_GET_DEFAULT_DESCRIPTION_MODES(
    _In_ IDDCX_MONITOR MonitorObject,
    _In_ const IDARG_IN_GETDEFAULTDESCRIPTIONMODES* pInArgs,
    _Out_ IDARG_OUT_GETDEFAULTDESCRIPTIONMODES* pOutArgs
);
typedef EVT_IDD_CX_MONITOR_GET_DEFAULT_DESCRIPTION_MODES
    *PFN_IDD_CX_MONITOR_GET_DEFAULT_DESCRIPTION_MODES;

// EVT_IDD_CX_MONITOR_ASSIGN_SWAPCHAIN
typedef
_Function_class_(EVT_IDD_CX_MONITOR_ASSIGN_SWAPCHAIN)
_IRQL_requires_same_
NTSTATUS
NTAPI
EVT_IDD_CX_MONITOR_ASSIGN_SWAPCHAIN(
    _In_ IDDCX_MONITOR MonitorObject,
    _In_ const IDARG_IN_SETSWAPCHAIN* pInArgs
);
typedef EVT_IDD_CX_MONITOR_ASSIGN_SWAPCHAIN *PFN_IDD_CX_MONITOR_ASSIGN_SWAPCHAIN;

// EVT_IDD_CX_MONITOR_UNASSIGN_SWAPCHAIN
typedef
_Function_class_(EVT_IDD_CX_MONITOR_UNASSIGN_SWAPCHAIN)
_IRQL_requires_same_
NTSTATUS
NTAPI
EVT_IDD_CX_MONITOR_UNASSIGN_SWAPCHAIN(
    _In_ IDDCX_MONITOR MonitorObject
);
typedef EVT_IDD_CX_MONITOR_UNASSIGN_SWAPCHAIN *PFN_IDD_CX_MONITOR_UNASSIGN_SWAPCHAIN;

// ---- Structure Definitions ----

// IDD_CX_CLIENT_CONFIG
struct IDD_CX_CLIENT_CONFIG {
    ULONG Size;
    PFN_IDD_CX_DEVICE_IO_CONTROL EvtIddCxDeviceIoControl;
    PFN_IDD_CX_PARSE_MONITOR_DESCRIPTION EvtIddCxParseMonitorDescription;
    PFN_IDD_CX_ADAPTER_INIT_FINISHED EvtIddCxAdapterInitFinished;
    PVOID EvtIddCxAdapterCommitModes;
    PFN_IDD_CX_MONITOR_GET_DEFAULT_DESCRIPTION_MODES EvtIddCxMonitorGetDefaultDescriptionModes;
    PVOID EvtIddCxMonitorQueryTargetModes;
    PFN_IDD_CX_MONITOR_ASSIGN_SWAPCHAIN EvtIddCxMonitorAssignSwapChain;
    PFN_IDD_CX_MONITOR_UNASSIGN_SWAPCHAIN EvtIddCxMonitorUnassignSwapChain;
    PVOID EvtIddCxMonitorI2CTransmit;
    PVOID EvtIddCxMonitorI2CReceive;
    PVOID EvtIddCxMonitorSetGammaRamp;
    PVOID EvtIddCxMonitorOPMGetCertificateSize;
    PVOID EvtIddCxMonitorOPMGetCertificate;
    PVOID EvtIddCxMonitorOPMCreateProtectedOutput;
    PVOID EvtIddCxMonitorOPMGetRandomNumber;
    PVOID EvtIddCxMonitorOPMSetSigningKeyAndSequenceNumbers;
    PVOID EvtIddCxMonitorOPMGetInformation;
    PVOID EvtIddCxMonitorOPMConfigureProtectedOutput;
    PVOID EvtIddCxMonitorOPMDestroyProtectedOutput;
    PVOID EvtIddCxMonitorGetPhysicalSize;
    PVOID EvtIddCxParseMonitorDescription2;
    PVOID EvtIddCxAdapterQueryTargetInfo;
    PVOID EvtIddCxAdapterCommitModes2;
    PVOID EvtIddCxMonitorSetDefaultHdrMetaData;
    PVOID EvtIddCxMonitorQueryTargetModes2;
};

// IDDCX_ENDPOINT_VERSION
struct IDDCX_ENDPOINT_VERSION {
    UINT Size;
    UINT MajorVer;
    UINT MinorVer;
    UINT Build;
    UINT64 SKU;
};

// IDDCX_ENDPOINT_DIAGNOSTIC_INFO
struct IDDCX_ENDPOINT_DIAGNOSTIC_INFO {
    UINT Size;
    IDDCX_TRANSMISSION_TYPE TransmissionType;
    PCWSTR pEndPointFriendlyName;
    PCWSTR pEndPointModelName;
    PCWSTR pEndPointManufacturerName;
    IDDCX_ENDPOINT_VERSION* pHardwareVersion;
    IDDCX_ENDPOINT_VERSION* pFirmwareVersion;
    IDDCX_FEATURE_IMPLEMENTATION GammaSupport;
};

// IDDCX_ADAPTER_CAPS
struct IDDCX_ADAPTER_CAPS {
    UINT Size;
    IDDCX_ADAPTER_FLAGS Flags;
    UINT64 MaxDisplayPipelineRate;
    UINT MaxMonitorsSupported;
    IDDCX_ENDPOINT_DIAGNOSTIC_INFO EndPointDiagnostics;
    UINT StaticDesktopReencodeFrameCount;
};

// IDARG_IN_ADAPTER_INIT
struct IDARG_IN_ADAPTER_INIT {
    WDFDEVICE WdfDevice;
    IDDCX_ADAPTER_CAPS* pCaps;
    PWDF_OBJECT_ATTRIBUTES ObjectAttributes;
};

// IDARG_OUT_ADAPTER_INIT
struct IDARG_OUT_ADAPTER_INIT {
    IDDCX_ADAPTER AdapterObject;
};

// IDARG_IN_ADAPTER_INIT_FINISHED
struct IDARG_IN_ADAPTER_INIT_FINISHED {
    NTSTATUS AdapterInitStatus;
};

// IDDCX_MONITOR_DESCRIPTION
struct IDDCX_MONITOR_DESCRIPTION {
    UINT Size;
    IDDCX_MONITOR_DESCRIPTION_TYPE Type;
    UINT DataSize;
    PVOID pData;
};

// IDDCX_MONITOR_INFO
struct IDDCX_MONITOR_INFO {
    UINT Size;
    DISPLAYCONFIG_VIDEO_OUTPUT_TECHNOLOGY MonitorType;
    UINT ConnectorIndex;
    IDDCX_MONITOR_DESCRIPTION MonitorDescription;
    GUID MonitorContainerId;
};

// IDARG_IN_MONITORCREATE
struct IDARG_IN_MONITORCREATE {
    PWDF_OBJECT_ATTRIBUTES ObjectAttributes;
    IDDCX_MONITOR_INFO* pMonitorInfo;
};

// IDARG_OUT_MONITORCREATE
struct IDARG_OUT_MONITORCREATE {
    IDDCX_MONITOR MonitorObject;
};

// IDARG_OUT_MONITORARRIVAL
struct IDARG_OUT_MONITORARRIVAL {
    LUID OsAdapterLuid;
    UINT OsTargetId;
};

// IDDCX_MONITOR_MODE
struct IDDCX_MONITOR_MODE {
    UINT Size;
    IDDCX_MONITOR_MODE_ORIGIN Origin;
    DISPLAYCONFIG_VIDEO_SIGNAL_INFO MonitorVideoSignalInfo;
};

// IDARG_IN_PARSEMONITORDESCRIPTION
struct IDARG_IN_PARSEMONITORDESCRIPTION {
    IDDCX_MONITOR_DESCRIPTION MonitorDescription;
    UINT MonitorModeBufferInputCount;
    IDDCX_MONITOR_MODE* pMonitorModes;
};

// IDARG_OUT_PARSEMONITORDESCRIPTION
struct IDARG_OUT_PARSEMONITORDESCRIPTION {
    UINT MonitorModeBufferOutputCount;
    UINT PreferredMonitorModeIdx;
};

#define NO_PREFERRED_MODE 0xffffffff

// IDARG_IN_GETDEFAULTDESCRIPTIONMODES
struct IDARG_IN_GETDEFAULTDESCRIPTIONMODES {
    UINT DefaultMonitorModeBufferInputCount;
    IDDCX_MONITOR_MODE* pDefaultMonitorModes;
};

// IDARG_OUT_GETDEFAULTDESCRIPTIONMODES
struct IDARG_OUT_GETDEFAULTDESCRIPTIONMODES {
    UINT DefaultMonitorModeBufferOutputCount;
    UINT PreferredMonitorModeIdx;
};

// IDARG_IN_SETSWAPCHAIN
struct IDARG_IN_SETSWAPCHAIN {
    IDDCX_SWAPCHAIN hSwapChain;
    HANDLE hNextSurfaceAvailable;
    LUID RenderAdapterLuid;
};

// IDDCX_METADATA
struct IDDCX_METADATA {
    UINT Size;
    UINT PresentationFrameNumber;
    UINT DirtyRectCount;
    UINT MoveRegionCount;
    BOOL HwProtectedSurface;
    UINT64 PresentDisplayQPCTime;
    IDXGIResource* pSurface;
};

// IDARG_IN_SWAPCHAINSETDEVICE
struct IDARG_IN_SWAPCHAINSETDEVICE {
    IDXGIDevice* pDevice;
};

// IDARG_OUT_RELEASEANDACQUIREBUFFER
struct IDARG_OUT_RELEASEANDACQUIREBUFFER {
    IDDCX_METADATA MetaData;
};

// ---- IddCx function prototypes ----
//
// The real IddCx functions are FORCEINLINE wrappers that dereference IddFunctions[].
// For kernel-mode builds we provide stubs in IddCxStubs.cpp.
//

_Must_inspect_result_
NTSTATUS
IddCxDeviceInitialize(
    _In_ WDFDEVICE Device
);

_Must_inspect_result_
NTSTATUS
IddCxAdapterInitAsync(
    _In_ CONST IDARG_IN_ADAPTER_INIT* pInArgs,
    _Out_ IDARG_OUT_ADAPTER_INIT* pOutArgs
);

_Must_inspect_result_
NTSTATUS
IddCxMonitorCreate(
    _In_ IDDCX_ADAPTER hAdapter,
    _In_ CONST IDARG_IN_MONITORCREATE* pInArgs,
    _Out_ IDARG_OUT_MONITORCREATE* pOutArgs
);

_Must_inspect_result_
NTSTATUS
IddCxMonitorArrival(
    _In_ IDDCX_MONITOR hMonitor,
    _Inout_ IDARG_OUT_MONITORARRIVAL* pOutArgs
);

_Must_inspect_result_
HRESULT
IddCxSwapChainSetDevice(
    _In_ IDDCX_SWAPCHAIN SwapChainObject,
    _In_ CONST IDARG_IN_SWAPCHAINSETDEVICE* pInArgs
);

_Must_inspect_result_
HRESULT
IddCxSwapChainReleaseAndAcquireBuffer(
    _In_ IDDCX_SWAPCHAIN SwapChainObject,
    _Out_ IDARG_OUT_RELEASEANDACQUIREBUFFER* pOutArgs
);

_Must_inspect_result_
HRESULT
IddCxSwapChainFinishedProcessingFrame(
    _In_ IDDCX_SWAPCHAIN SwapChainObject
);
