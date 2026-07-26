import 'dart:typed_data';

import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import '../services/connection_service.dart';

class DisplayView extends StatefulWidget {
  const DisplayView({super.key});

  @override
  State<DisplayView> createState() => _DisplayViewState();
}

class _DisplayViewState extends State<DisplayView> {
  void _sendTouchEvent(String eventType, Offset localPosition, Size size) {
    if (size == Size.zero) return;
    final service = context.read<ConnectionService>();
    final nx = (localPosition.dx / size.width).clamp(0.0, 1.0);
    final ny = (localPosition.dy / size.height).clamp(0.0, 1.0);
    service.sendTouchEvent(eventType, nx, ny);
  }

  @override
  Widget build(BuildContext context) {
    final frame = context.select<ConnectionService, Uint8List?>(
      (service) => service.currentFrame,
    );

    if (frame == null) {
      return const ColoredBox(
        color: Colors.black,
        child: Center(
          child: SizedBox.square(
            dimension: 36,
            child: CircularProgressIndicator(strokeWidth: 3),
          ),
        ),
      );
    }

    return ColoredBox(
      color: Colors.black,
      child: LayoutBuilder(
        builder: (context, constraints) {
          final size = Size(constraints.maxWidth, constraints.maxHeight);
          return GestureDetector(
            onPanStart: (details) =>
                _sendTouchEvent('down', details.localPosition, size),
            onPanUpdate: (details) =>
                _sendTouchEvent('move', details.localPosition, size),
            onPanEnd: (_) => _sendTouchEvent('up', Offset.zero, size),
            onTapUp: (details) {
              _sendTouchEvent('down', details.localPosition, size);
              Future.delayed(const Duration(milliseconds: 50), () {
                _sendTouchEvent('up', Offset.zero, size);
              });
            },
            child: Center(
              child: Image.memory(
                frame,
                fit: BoxFit.contain,
                gaplessPlayback: true,
                filterQuality: FilterQuality.medium,
                errorBuilder: (context, error, stackTrace) {
                  return const Center(
                    child: Icon(
                      Icons.broken_image_outlined,
                      color: Colors.white54,
                      size: 44,
                    ),
                  );
                },
              ),
            ),
          );
        },
      ),
    );
  }
}
