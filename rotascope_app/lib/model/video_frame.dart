import 'dart:typed_data';

enum VideoCodec { jpeg, h264 }

class VideoFrame {
  final Uint8List data;
  final int width;
  final int height;
  final int displayIndex;
  final int timestamp;
  final VideoCodec codec;

  const VideoFrame({
    required this.data,
    required this.width,
    required this.height,
    this.displayIndex = 0,
    this.timestamp = 0,
    this.codec = VideoCodec.jpeg,
  });

  bool get isKeyFrame => codec == VideoCodec.h264 && _isH264KeyFrame(data);

  static bool _isH264KeyFrame(Uint8List data) {
    int i = 0;
    while (i < data.length - 4) {
      if (data[i] == 0 && data[i + 1] == 0 && data[i + 2] == 0 && data[i + 3] == 1) {
        final nalType = data[i + 4] & 0x1F;
        if (nalType == 7 || nalType == 8) return true;
        if (nalType >= 1 && nalType <= 5) return nalType == 5;
        i += 5;
      } else if (data[i] == 0 && data[i + 1] == 0 && data[i + 2] == 1) {
        final nalType = data[i + 3] & 0x1F;
        if (nalType == 7 || nalType == 8) return true;
        if (nalType >= 1 && nalType <= 5) return nalType == 5;
        i += 4;
      } else {
        i++;
      }
    }
    return false;
  }
}
