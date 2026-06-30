#include "Driver.h"

extern "C"
void SubmitFrame(PVOID buffer, SIZE_T size)
{
    DbgPrint("Frame received: %llu bytes\n", size);

    // 这里是关键出口：
    // ↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓

    // 1. 写共享内存
    // 2. 交给 user-mode service
    // 3. NVENC编码
    // 4. USB发送到手机
}