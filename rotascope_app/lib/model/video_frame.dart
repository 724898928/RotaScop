
// 视频帧数据模型
import 'package:flutter/foundation.dart';

class VideoFrame {
  final int display_index;
  final Uint8List data;
  final int width;
  final int height;
  final int timestamp;

  VideoFrame({
    required this.display_index,
    required this.data,
    required this.width,
    required this.height,
    required this.timestamp,
  });

  factory VideoFrame.fromJson(Map<String, dynamic> json) {
    return VideoFrame(
      display_index: json['display_index'],
      data: json['data'],
      width: json['width'],
      height: json['height'],
      timestamp: json['timestamp'],
    );
  }
}