import 'dart:typed_data';

import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'dart:ui' as ui;
import '../services/connection_service.dart';

class DisplayView extends StatefulWidget {
  const DisplayView({super.key});

  @override
  State<DisplayView> createState() => _DisplayViewState();
}

class _DisplayViewState extends State<DisplayView> {
  ui.Image? _currentImage;

  @override
  Widget build(BuildContext context) {
    final connectionService = Provider.of<ConnectionService>(context);
    final frame = connectionService.currentFrame;

    if (frame == null) {
      return const Center(
        child: CircularProgressIndicator(),
      );
    }

    return FutureBuilder<ui.Image>(
      future: _loadImage(frame),
      builder: (context, snapshot) {
        if (snapshot.hasData) {
          _currentImage = snapshot.data;
          return CustomPaint(
            painter: DisplayPainter(_currentImage!),
            size: Size.infinite,
          );
        } else if (snapshot.hasError) {
          return Center(
            child: Text('Error: ${snapshot.error}'),
          );
        } else {
          return const Center(
            child: CircularProgressIndicator(),
          );
        }
      },
    );
  }

  Future<ui.Image> _loadImage(List<int> jpegData) async {
    final codec = await ui.instantiateImageCodec(
      Uint8List.fromList(jpegData),
    );
    final frame = await codec.getNextFrame();
    return frame.image;
  }
}

class DisplayPainter extends CustomPainter {
  final ui.Image image;

  DisplayPainter(this.image);

  @override
  void paint(Canvas canvas, Size size) {
    final paint = Paint();
    
    // 计算适应屏幕的尺寸
    final imageRatio = image.width / image.height;
    final screenRatio = size.width / size.height;
    
    Rect destRect;
    if (screenRatio > imageRatio) {
      // 屏幕更宽，按高度适应
      final height = size.height;
      final width = height * imageRatio;
      final left = (size.width - width) / 2;
      destRect = Rect.fromLTWH(left, 0, width, height);
    } else {
      // 屏幕更高，按宽度适应
      final width = size.width;
      final height = width / imageRatio;
      final top = (size.height - height) / 2;
      destRect = Rect.fromLTWH(0, top, width, height);
    }
    
    canvas.drawImageRect(image, 
      Rect.fromLTWH(0, 0, image.width.toDouble(), image.height.toDouble()),
      destRect, 
      paint
    );
  }

  @override
  bool shouldRepaint(covariant CustomPainter oldDelegate) {
    return true;
  }
}