#define WIN32_LEAN_AND_MEAN
#include <windows.h>
#include <winioctl.h>
#include <winsock2.h>
#include <ws2tcpip.h>
#include <stdio.h>
#include <stdlib.h>

#pragma comment(lib, "ws2_32.lib")

#define ROTASCOPE_DEVICE_PATH L"\\\\.\\RotaScope"
#define ROTASCOPE_EVENT_NAME  L"Global\\RotaScopeFrameEvent"
#define DEFAULT_SERVER_PORT   8080
#define DEFAULT_SERVER_ADDR   "127.0.0.1"

#define IOCTL_ROTASCOPE_GET_FRAME \
    CTL_CODE(FILE_DEVICE_UNKNOWN, 0x800, METHOD_BUFFERED, FILE_READ_ACCESS | FILE_WRITE_ACCESS)

#define IOCTL_ROTASCOPE_WAIT_FRAME \
    CTL_CODE(FILE_DEVICE_UNKNOWN, 0x801, METHOD_BUFFERED, FILE_READ_ACCESS)

#pragma pack(push, 1)
typedef struct _SHARED_FRAME {
    volatile LONG   FrameReady;
    volatile LONG   Width;
    volatile LONG   Height;
    volatile LONG   Stride;
    UCHAR           Buffer[1920 * 1080 * 4];
} SHARED_FRAME;
#pragma pack(pop)

static SOCKET g_ServerSocket = INVALID_SOCKET;

static int
ConnectToServer(const char* addr, int port)
{
    struct sockaddr_in serverAddr;

    g_ServerSocket = socket(AF_INET, SOCK_STREAM, 0);

    if (g_ServerSocket == INVALID_SOCKET)
    {
        printf("[RotaScopeCompanion] Failed to create socket: %d\n", WSAGetLastError());
        return -1;
    }

    serverAddr.sin_family = AF_INET;
    serverAddr.sin_port = htons((u_short)port);
    inet_pton(AF_INET, addr, &serverAddr.sin_addr);

    if (connect(g_ServerSocket, (struct sockaddr*)&serverAddr, sizeof(serverAddr)) == SOCKET_ERROR)
    {
        printf("[RotaScopeCompanion] Failed to connect to %s:%d: %d\n",
               addr, port, WSAGetLastError());
        closesocket(g_ServerSocket);
        g_ServerSocket = INVALID_SOCKET;
        return -1;
    }

    printf("[RotaScopeCompanion] Connected to server %s:%d\n", addr, port);

    return 0;
}

static int
SendFrame(SHARED_FRAME* frame)
{
    DWORD payloadSize;
    DWORD totalSent;
    int result;

    if (g_ServerSocket == INVALID_SOCKET)
    {
        return -1;
    }

    payloadSize = frame->Stride * frame->Height;

    if (payloadSize == 0 || payloadSize > sizeof(frame->Buffer))
    {
        return -1;
    }

    {
        UINT32 width = htonl((UINT32)frame->Width);
        UINT32 height = htonl((UINT32)frame->Height);

        send(g_ServerSocket, (const char*)&width, sizeof(width), 0);
        send(g_ServerSocket, (const char*)&height, sizeof(height), 0);
    }

    totalSent = 0;

    while (totalSent < payloadSize)
    {
        result = send(g_ServerSocket,
                      (const char*)(frame->Buffer + totalSent),
                      payloadSize - totalSent, 0);

        if (result == SOCKET_ERROR)
        {
            printf("[RotaScopeCompanion] Send error: %d\n", WSAGetLastError());
            closesocket(g_ServerSocket);
            g_ServerSocket = INVALID_SOCKET;
            return -1;
        }

        totalSent += result;
    }

    return 0;
}

int
main(int argc, char* argv[])
{
    HANDLE deviceHandle;
    HANDLE frameEvent;
    WSADATA wsaData;
    const char* serverAddr;
    int serverPort;
    int retryCount;

    serverAddr = DEFAULT_SERVER_ADDR;
    serverPort = DEFAULT_SERVER_PORT;

    if (argc > 1) serverAddr = argv[1];
    if (argc > 2) serverPort = atoi(argv[2]);

    printf("RotaScope Companion Service\n");
    printf("===========================\n");
    printf("Server: %s:%d\n", serverAddr, serverPort);

    if (WSAStartup(MAKEWORD(2, 2), &wsaData) != 0)
    {
        printf("WSAStartup failed\n");
        return 1;
    }

    deviceHandle = CreateFileW(
        ROTASCOPE_DEVICE_PATH,
        GENERIC_READ | GENERIC_WRITE,
        0,
        NULL,
        OPEN_EXISTING,
        FILE_ATTRIBUTE_NORMAL,
        NULL
    );

    if (deviceHandle == INVALID_HANDLE_VALUE)
    {
        printf("Failed to open device %ls: %d\n", ROTASCOPE_DEVICE_PATH, GetLastError());
        printf("Is the RotaScope driver installed?\n");
        WSACleanup();
        return 1;
    }

    frameEvent = OpenEventW(EVENT_ALL_ACCESS, FALSE, ROTASCOPE_EVENT_NAME);

    if (frameEvent == NULL)
    {
        printf("Failed to open frame event: %d\n", GetLastError());
        CloseHandle(deviceHandle);
        WSACleanup();
        return 1;
    }

    printf("Driver device opened, waiting for frames...\n");

    retryCount = 0;

    while (TRUE)
    {
        SHARED_FRAME frame;
        DWORD bytesReturned;

        WaitForSingleObject(frameEvent, INFINITE);

        if (DeviceIoControl(
                deviceHandle,
                IOCTL_ROTASCOPE_GET_FRAME,
                NULL, 0,
                &frame, sizeof(frame),
                &bytesReturned,
                NULL))
        {
            if (frame.FrameReady && bytesReturned >= sizeof(SHARED_FRAME))
            {
                if (g_ServerSocket == INVALID_SOCKET)
                {
                    if (ConnectToServer(serverAddr, serverPort) != 0)
                    {
                        retryCount++;

                        if (retryCount > 10)
                        {
                            Sleep(5000);
                            retryCount = 0;
                        }

                        continue;
                    }
                }

                if (SendFrame(&frame) != 0)
                {
                    retryCount++;
                    continue;
                }

                retryCount = 0;
            }
        }
    }

    CloseHandle(frameEvent);
    CloseHandle(deviceHandle);
    WSACleanup();

    return 0;
}
