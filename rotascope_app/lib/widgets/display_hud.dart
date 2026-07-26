import 'package:flutter/material.dart';

class DisplayHUD extends StatelessWidget {
  const DisplayHUD({
    super.key,
    required this.currentDisplay,
    required this.totalDisplays,
    required this.fps,
    required this.rotation,
  });

  final int currentDisplay;
  final int totalDisplays;
  final double fps;
  final double rotation;

  @override
  Widget build(BuildContext context) {
    final displayCount = totalDisplays < 1 ? 1 : totalDisplays;

    return Center(
      child: DecoratedBox(
        decoration: BoxDecoration(
          color: Colors.black.withValues(alpha: 0.58),
          borderRadius: BorderRadius.circular(8),
          border: Border.all(color: Colors.white24),
        ),
        child: Padding(
          padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 7),
          child: DefaultTextStyle(
            style: const TextStyle(
              color: Colors.white,
              fontSize: 13,
            ),
            child: Row(
              mainAxisSize: MainAxisSize.min,
              children: [
                const Icon(Icons.desktop_windows, size: 16),
                const SizedBox(width: 6),
                Text('显示器 ${currentDisplay + 1}/$displayCount'),
                const SizedBox(width: 14),
                const Icon(Icons.speed, size: 16),
                const SizedBox(width: 6),
                Text('${fps.toStringAsFixed(0)} FPS'),
                const SizedBox(width: 14),
                const Icon(Icons.screen_rotation_alt, size: 16),
                const SizedBox(width: 6),
                Text(rotation.toStringAsFixed(1)),
              ],
            ),
          ),
        ),
      ),
    );
  }
}
