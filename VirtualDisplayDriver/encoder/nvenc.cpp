#include <nvEncodeAPI.h>
#include <d3d11.h>
#include <iostream>

class NVENCEncoder {
private:
    void* m_EncoderHandle = nullptr;
    NV_ENCODE_API_FUNCTION_LIST m_NvApi;

public:
    bool Initialize(ID3D11Device* d3d11Device) {
        // Load NVIDIA encoding library
        HMODULE hModule = LoadLibraryA("nvEncodeAPI64.dll");
        if (!hModule) return false;

        typedef NVENCSTATUS(NVENC_API* NvEncodeAPICreateInstanceFunction)(NV_ENCODE_API_FUNCTION_LIST*);
        auto NvEncodeAPICreateInstance = (NvEncodeAPICreateInstanceFunction)GetProcAddress(hModule, "NvEncodeAPICreateInstance");
        
        m_NvApi.version = NV_ENCODE_API_FUNCTION_LIST_VER;
        NvEncodeAPICreateInstance(&m_NvApi);

        // Open encoding session
        NV_ENC_OPEN_ENCODE_SESSION_EX_PARAMS openParams = {};
        openParams.version = NV_ENC_OPEN_ENCODE_SESSION_EX_PARAMS_VER;
        openParams.device = d3d11Device;
        openParams.deviceType = NV_ENC_DEVICE_TYPE_DIRECTX;
        openParams.apiVersion = NVENCAPI_VERSION;

        m_NvApi.nvEncOpenEncodeSessionEx(&openParams, &m_EncoderHandle);
        return ConfigureLowLatency();
    }

    bool ConfigureLowLatency() {
        NV_ENC_CONFIG encodeConfig = {};
        encodeConfig.version = NV_ENC_CONFIG_VER;
        
        // CRITICAL INDUSTRIAL LOW LATENCY PARAMETERS:
        encodeConfig.profileGUID = NV_ENC_H264_PROFILE_HIGH_GUID;
        encodeConfig.gopLength = 1;                     // Infinite GOP (I-Frame stream fallback)
        encodeConfig.frameIntervalP = 1;                // B-Frames = OFF
        
        encodeConfig.rcParams.rateControlMode = NV_ENC_PARAMS_RC_CBR; // Constant Bitrate
        encodeConfig.rcParams.averageBitrate = 15000000;              // 15 Mbps target
        encodeConfig.rcParams.enableMinQP = 1;
        
        // Zero-latency optimization flags
        encodeConfig.encodeCodecConfig.h264Config.idrPeriod = 1;
        encodeConfig.encodeCodecConfig.h264Config.enableIntraRefresh = 1; // Row-by-row intra refreshes
        encodeConfig.encodeCodecConfig.h264Config.intraRefreshPeriod = 30;

        NV_ENC_INITIALIZE_PARAMS initParams = {};
        initParams.version = NV_ENC_INITIALIZE_PARAMS_VER;
        initParams.encodeGUID = NV_ENC_CODEC_H264_GUID;
        initParams.presetGUID = NV_ENC_PRESET_LOW_LATENCY_HQ_GUID; // Ultra low-latency preset
        initParams.encodeWidth = 1920;
        initParams.encodeHeight = 1080;
        initParams.encodeConfig = &encodeConfig;

        NVENCSTATUS status = m_NvApi.nvEncInitializeEncoder(m_EncoderHandle, &initParams);
        return status == NV_ENC_SUCCESS;
    }

    void EncodeFrame(ID3D11Texture2D* gpuTexture, void* outBitstreamBuffer) {
        NV_ENC_PIC_PARAMS params = {};
        params.version = NV_ENC_PIC_PARAMS_VER;
        params.inputWidth = 1920;
        params.inputHeight = 1080;
        
        // Set direct GPU texture surface
        params.inputBuffer = gpuTexture;
        params.outputBitstream = outBitstreamBuffer;
        
        // Instruct encoder to skip internal queue buffering (immediate hardware flush)
        params.encodePicFlags = NV_ENC_PIC_FLAG_LOW_LATENCY | NV_ENC_PIC_FLAG_FORCEIDR;
        
        m_NvApi.nvEncEncodeFrame(m_EncoderHandle, &params);
    }
};
