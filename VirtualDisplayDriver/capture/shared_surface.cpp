#include <d3d11.h>
#include <iostream>

HANDLE MapSharedSurface(ID3D11Device* device, ID3D11Texture2D* texture) {
    IDXGIResource* tempResource = nullptr;
    HRESULT hr = texture->QueryInterface(__uuidof(IDXGIResource), (void**)&tempResource);
    if (FAILED(hr)) return nullptr;

    HANDLE sharedHandle = nullptr;
    hr = tempResource->GetSharedHandle(&sharedHandle);
    tempResource->Release();

    if (FAILED(hr)) {
        std::cerr << "Failed to obtain shared D3D11 handle for GPU Zero-Copy" << std::endl;
        return nullptr;
    }
    return sharedHandle; // Direct hardware address, zero CPU cache impact
}