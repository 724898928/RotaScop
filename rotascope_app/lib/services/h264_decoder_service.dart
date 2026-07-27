import 'dart:async';
import 'dart:typed_data';

import 'package:flutter/foundation.dart';
import 'package:flutter/services.dart';

class H264DecoderService extends ChangeNotifier {
  static const _channel = MethodChannel('rotascope/h264');

  Uint8List? _decodedFrame;
  int _frameWidth = 0;
  int _frameHeight = 0;
  bool _initialized = false;
  String? _lastError;

  Uint8List? get decodedFrame => _decodedFrame;
  int get frameWidth => _frameWidth;
  int get frameHeight => _frameHeight;
  bool get isAvailable => _initialized;
  String? get lastError => _lastError;

  Future<bool> initDecoder(int width, int height) async {
    if (_initialized) return true;

    try {
      final result = await _channel.invokeMethod<bool>('init', {
        'width': width,
        'height': height,
        'mime': 'video/avc',
      });
      _initialized = result ?? false;
      if (_initialized) notifyListeners();
      return _initialized;
    } on MissingPluginException {
      _lastError = 'H.264 decoder platform channel not available';
      if (kDebugMode) debugPrint(_lastError);
      return false;
    } catch (e) {
      _lastError = e.toString();
      if (kDebugMode) debugPrint('H.264 init error: $e');
      return false;
    }
  }

  Future<Uint8List?> decodeFrame(Uint8List h264Data) async {
    if (!_initialized) return null;

    try {
      final result = await _channel.invokeMethod<Uint8List>('decode', {
        'data': h264Data,
      });

      if (result != null) {
        _decodedFrame = result;
        notifyListeners();
      }
      return result;
    } on MissingPluginException {
      return null;
    } catch (e) {
      if (kDebugMode) debugPrint('H.264 decode error: $e');
      return null;
    }
  }

  Future<void> release() async {
    if (!_initialized) return;
    try {
      await _channel.invokeMethod<void>('release');
    } catch (_) {}
    _initialized = false;
    _decodedFrame = null;
    notifyListeners();
  }

  @override
  void dispose() {
    release();
    super.dispose();
  }
}
