import 'dart:typed_data';

import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import '../services/connection_service.dart';

class DisplayView extends StatelessWidget {
  const DisplayView({super.key});

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
  }
}
