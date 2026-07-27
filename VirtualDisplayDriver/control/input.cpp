#include <windows.h>
#include <iostream>

#pragma pack(push, 1)
struct TouchInputPacket {
    uint8_t type;       // 0: Down, 1: Move, 2: Up
    uint16_t x;         // 0 to 65535 normalized coordinates
    uint16_t y;
};
#pragma pack(pop)

class WindowsInputController {
public:
    void InjectTouchEvent(TouchInputPacket packet) {
        INPUT input = {};
        input.type = INPUT_MOUSE;
        
        // Absolute virtual desktop mapping
        input.mi.dx = packet.x; 
        input.mi.dy = packet.y;
        
        // Target virtual second screen coordinate space
        input.mi.dwFlags = MOUSEEVENTF_ABSOLUTE | MOUSEEVENTF_VIRTUALDESK;

        switch (packet.type) {
            case 0: // TOUCH_DOWN
                input.mi.dwFlags |= MOUSEEVENTF_LEFTDOWN | MOUSEEVENTF_MOVE;
                break;
            case 1: // TOUCH_MOVE
                input.mi.dwFlags |= MOUSEEVENTF_MOVE;
                break;
            case 2: // TOUCH_UP
                input.mi.dwFlags |= MOUSEEVENTF_LEFTUP;
                break;
        }

        // Injected straight into OS message loop
        SendInput(1, &input, sizeof(INPUT));
    }
};
