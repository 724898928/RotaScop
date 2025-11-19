import 'package:flutter/material.dart';

class DisplayHUD extends StatelessWidget {
  final int currentDisplay;
  final int totalDisplays;
  final double rotation;

  const DisplayHUD({
    super.key,
    required this.currentDisplay,
    required this.totalDisplays,
    required this.rotation,
  });

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.all(16),
      child: Column(
        children: [
          Text(
            '显示器 ${currentDisplay + 1}/$totalDisplays',
            style: const TextStyle(
              color: Colors.white,
              fontSize: 18,
              fontWeight: FontWeight.bold,
            ),
          ),
          const SizedBox(height: 8),
          Text(
            '旋转: ${rotation.toStringAsFixed(1)}°',
            style: const TextStyle(
              color: Colors.white70,
              fontSize: 14,
            ),
          ),
          const SizedBox(height: 8),
          // 旋转指示器
          Container(
            width: 200,
            height: 4,
            decoration: BoxDecoration(
              color: Colors.white24,
              borderRadius: BorderRadius.circular(2),
            ),
            child: Stack(
              children: [
                // 中心点
                Positioned(
                  left: 100,
                  child: Container(
                    width: 8,
                    height: 8,
                    decoration: const BoxDecoration(
                      color: Colors.white,
                      shape: BoxShape.circle,
                    ),
                  ),
                ),
                // 旋转指示
                Positioned(
                  left: 100 + (rotation.clamp(-30.0, 30.0) / 30.0 * 100),
                  child: Container(
                    width: 12,
                    height: 12,
                    decoration: BoxDecoration(
                      color: Colors.blue,
                      shape: BoxShape.circle,
                      border: Border.all(color: Colors.white, width: 2),
                    ),
                  ),
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }
}