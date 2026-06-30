#pragma once
#define FRAME_WIDTH 1920
#define FRAME_HEIGHT 1080
#define BYTES_PER_PIXEL 4


struct SharedFrame
{
volatile LONG frameIndex;
BYTE pixels[FRAME_WIDTH * FRAME_HEIGHT * BYTES_PER_PIXEL];
};