
#include <d3d11.h>
#include <dxgi1_2.h>
#include <iostream>

class DXGICapture {
private:
    ID3D11Device*           m_Device = nullptr;
    ID3D11DeviceContext*    m_Context = nullptr;
    IDXGIOutputDuplication* m_Dupl = nullptr;
    
public:
    bool Initialize() {
        // Initialize D3D11 Device & Context
        D3D_FEATURE_LEVEL featureLevel;
        HRESULT hr = D3D11CreateDevice(nullptr, D3D_DRIVER_TYPE_HARDWARE, nullptr, 
                                       D3D11_CREATE_DEVICE_VIDEO_SUPPORT, nullptr, 0, 
                                       D3D11_SDK_VERSION, &m_Device, &featureLevel, &m_Context);
        if (FAILED(hr)) return false;

        // Obtain DXGI output duplication handle from Virtual Monitor output
        IDXGIDevice* dxgiDevice = nullptr;
        m_Device->QueryInterface(__uuidof(IDXGIDevice), (void**)&dxgiDevice);
        
        IDXGIAdapter* dxgiAdapter = nullptr;
        dxgiDevice->GetParent(__uuidof(IDXGIAdapter), (void**)&dxgiAdapter);
        dxgiDevice->Release();

        IDXGIOutput* dxgiOutput = nullptr;
        // Select virtual display monitor output (typically index 1 or 2)
        if (FAILED(dxgiAdapter->EnumOutputs(1, &dxgiOutput))) {
            dxgiAdapter->EnumOutputs(0, &dxgiOutput); // Fallback to main
        }
        dxgiAdapter->Release();

        IDXGIOutput1* dxgiOutput1 = nullptr;
        dxgiOutput->QueryInterface(__uuidof(IDXGIOutput1), (void**)&dxgiOutput1);
        dxgiOutput->Release();

        // Create duplication interface
        hr = dxgiOutput1->DuplicateOutput(m_Device, &m_Dupl);
        dxgiOutput1->Release();

        return SUCCEEDED(hr);
    }

    ID3D11Texture2D* CaptureFrame(DXGI_OUTDUPL_FRAME_INFO* outFrameInfo) {
        if (!m_Dupl) return nullptr;

        IDXGIResource* desktopResource = nullptr;
        m_Dupl->ReleaseFrame(); // Release previous frame lock immediately
        
        // Zero timeout to keep loop non-blocking (GPU rendering sync)
        HRESULT hr = m_Dupl->AcquireNextFrame(0, outFrameInfo, &desktopResource);
        if (FAILED(hr)) {
            return nullptr; // No update this cycle (keep thread active)
        }

        ID3D11Texture2D* desktopTexture = nullptr;
        hr = desktopResource->QueryInterface(__uuidof(ID3D11Texture2D), (void**)&desktopTexture);
        desktopResource->Release();

        if (FAILED(hr)) return nullptr;
        return desktopTexture; // Zero-copy texture handle on GPU
    }
};