
import 'package:flutter/foundation.dart';

class VideoFrame {
  final int displayIndex;
  final Uint8List data;
  final int width;
  final int height;
  final int timestamp;

  VideoFrame({
    required this.displayIndex,
    required this.data,
    required this.width,
    required this.height,
    required this.timestamp,
  });

  factory VideoFrame.fromJson(Map<String, dynamic> json) {
    return VideoFrame(
      displayIndex: json['display_index'],
      data: json['data'],
      width: json['width'],
      height: json['height'],
      timestamp: json['timestamp'],
    );
  }
}