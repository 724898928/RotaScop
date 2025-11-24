import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'dart:ui' as ui;
import 'dart:typed_data';
import '../services/connection_service.dart';

class DisplayView extends StatefulWidget {
  const DisplayView({super.key});

  @override
  State<DisplayView> createState() => _DisplayViewState();
}

class _DisplayViewState extends State<DisplayView> {
  ui.Image? _currentImage;
  bool _isLoading = false;
  bool _hasError = false;
  String _errorMessage = '';

  @override
  Widget build(BuildContext context) {
    final connectionService = Provider.of<ConnectionService>(context);
    final frame = connectionService.currentFrame;

    if (_hasError) {
      return Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            const Icon(
              Icons.error_outline,
              color: Colors.red,
              size: 48,
            ),
            const SizedBox(height: 16),
            Text(
              _errorMessage,
              style: const TextStyle(color: Colors.white),
              textAlign: TextAlign.center,
            ),
            const SizedBox(height: 16),
            ElevatedButton(
              onPressed: () {
                setState(() {
                  _hasError = false;
                  _errorMessage = '';
                });
              },
              child: const Text('重试'),
            ),
          ],
        ),
      );
    }

    if (frame == null || frame.isEmpty || _isLoading) {
      return Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            const CircularProgressIndicator(),
            const SizedBox(height: 16),
            Text(
              '显示器 ${connectionService.currentDisplay + 1}',
              style: const TextStyle(
                color: Colors.white,
                fontSize: 16,
              ),
            ),
            const SizedBox(height: 8),
            const Text(
              '等待图像数据...',
              style: TextStyle(
                color: Colors.white70,
                fontSize: 12,
              ),
            ),
          ],
        ),
      );
    }

    // 使用 Image.memory 显示 Uint8List 数据
    return Center(
      child: Image.memory(
        frame as Uint8List,
        fit: BoxFit.contain,
        errorBuilder: (context, error, stackTrace) {
          // 只设置一次错误状态，避免循环
          if (!_hasError) {
            WidgetsBinding.instance.addPostFrameCallback((_) {
              if (mounted) {
                setState(() {
                  _hasError = true;
                  _errorMessage = '图像解码失败: $error';
                });
              }
            });
          }
          return Column(
            mainAxisAlignment: MainAxisAlignment.center,
            children: [
              const Icon(Icons.broken_image, color: Colors.red, size: 48),
              const SizedBox(height: 16),
              Text('解码错误: $error', style: const TextStyle(color: Colors.white)),
            ],
          );
        },
      ),
    );
  }
}