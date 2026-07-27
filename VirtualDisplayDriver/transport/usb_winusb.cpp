
#include <windows.h>
#include <winusb.h>
#include <iostream>

class WinUSBTransport {
private:
    HANDLE          m_DeviceHandle = INVALID_HANDLE_VALUE;
    WINUSB_INTERFACE_HANDLE m_WinusbHandle = INVALID_HANDLE_VALUE;
    UCHAR           m_BulkOutPipe = 0x01; // Outbound Endpoint

public:
    bool OpenDevice(const char* devicePath) {
        m_DeviceHandle = CreateFileA(devicePath, GENERIC_READ | GENERIC_WRITE, 
                                     FILE_SHARE_READ | FILE_SHARE_WRITE, nullptr, 
                                     OPEN_EXISTING, FILE_FLAG_OVERLAPPED, nullptr);
        if (m_DeviceHandle == INVALID_HANDLE_VALUE) return false;

        BOOL ok = WinUsb_Initialize(m_DeviceHandle, &m_WinusbHandle);
        if (!ok) {
            CloseHandle(m_DeviceHandle);
            return false;
        }

        // Configure USB Pipe policies for ultra-low latency
        ULONG timeout = 5; // 5ms write timeout
        WinUsb_SetPipePolicy(m_WinusbHandle, m_BulkOutPipe, PIPE_TRANSFER_TIMEOUT, sizeof(timeout), &timeout);
        
        // Disable queuing inside WinUSB driver to avoid frame stacking
        UCHAR rawIo = 1;
        WinUsb_SetPipePolicy(m_WinusbHandle, m_BulkOutPipe, RAW_IO, sizeof(rawIo), &rawIo);

        return true;
    }

    bool SendVideoFrame(uint8_t* h264Data, ULONG dataSize) {
        if (m_WinusbHandle == INVALID_HANDLE_VALUE) return false;

        ULONG bytesWritten = 0;
        // Direct write bypasses TCP/IP network protocol stack, writing straight to hardware bus
        BOOL ok = WinUsb_WritePipe(m_WinusbHandle, m_BulkOutPipe, h264Data, dataSize, &bytesWritten, nullptr);
        return ok && (bytesWritten == dataSize);
    }
};