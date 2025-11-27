import 'dart:convert';
import 'dart:typed_data';
import 'package:flutter/foundation.dart';
import 'package:web_socket_channel/web_socket_channel.dart';

enum ConnectionStatus { disconnected, connecting, connected, error }

class ConnectionService extends ChangeNotifier {
  WebSocketChannel? _channel;
  ConnectionStatus _status = ConnectionStatus.disconnected;
  String _serverAddress = '192.168.31.197:8080';

  int _currentDisplay = 0;
  int _totalDisplays = 3;

  Uint8List? _currentFrame;
  int _frameWidth = 1280;
  int _frameHeight = 720;

  // 添加帧统计
  int _totalFramesReceived = 0;
  int _validFramesReceived = 0;
  int _invalidFramesReceived = 0;

  ConnectionStatus get status => _status;
  String get serverAddress => _serverAddress;
  bool get isConnected => _status == ConnectionStatus.connected;
  int get currentDisplay => _currentDisplay;
  int get totalDisplays => _totalDisplays;
  Uint8List? get currentFrame => _currentFrame;
  int get frameWidth => _frameWidth;
  int get frameHeight => _frameHeight;

  void updateServerAddress(String address) {
    _serverAddress = address;
    notifyListeners();
  }

  Future<void> connect() async {
    if (_status == ConnectionStatus.connected) return;

    _status = ConnectionStatus.connecting;
    notifyListeners();

    try {
      _channel = WebSocketChannel.connect(
        Uri.parse('ws://$_serverAddress'),
      );

      _channel!.stream.listen(
        _handleMessage,
        onError: _handleError,
        onDone: _handleDisconnect,
      );

      _status = ConnectionStatus.connected;
      notifyListeners();
    } catch (e) {
      _status = ConnectionStatus.error;
      notifyListeners();
      rethrow;
    }
  }

  void disconnect() {
    _channel?.sink.close();
    _channel = null;
    _status = ConnectionStatus.disconnected;
    _currentFrame = null;
    notifyListeners();
  }

  void _handleMessage(dynamic message) {
    _totalFramesReceived++;
    try {
      if (message is String) {
        _handleTextMessage(message);
      } else if (message is Uint8List) {
        _handleBinaryMessage(message);
      } else {
        if (kDebugMode) {
          print('Unknown message type: ${message.runtimeType}');
        }
      }
    } catch (e) {
      _invalidFramesReceived++;
      if (kDebugMode) {
        print('Error handling message: $e');
      }
    }
  }

  void _handleTextMessage(String message) {
    try {
      final data = jsonDecode(message);

      if (data['type'] == 'DisplayConfig') {
        _handleDisplayConfig(data);
      } else if (data['type'] == 'Heartbeat') {
        if (kDebugMode) {
          print('Heartbeat received');
        }
      } else if (data['type'] == 'Error') {
        if (kDebugMode) {
          print('Server error: ${data['message']}');
        }
      }
    } catch (e) {
      if (kDebugMode) {
        print('Error parsing text message: $e');
      }
    }
  }

  void _handleBinaryMessage(Uint8List message) {
    try {
      // 增强的数据验证
      if (!_isValidImageData(message)) {
        _invalidFramesReceived++;
        if (kDebugMode) {
          print('Invalid image data received');
        }
        return;
      }

      _validFramesReceived++;
      _currentFrame = message;

      if (kDebugMode) {
        print('Valid frame received: ${message.length} bytes, '
            'Stats: $_validFramesReceived/$_totalFramesReceived valid, '
            '${(_validFramesReceived / _totalFramesReceived * 100).toStringAsFixed(1)}% success rate');
      }

      notifyListeners();
    } catch (e) {
      _invalidFramesReceived++;
      if (kDebugMode) {
        print('Error handling binary message: $e');
      }
    }
  }

  bool _isValidImageData(Uint8List data) {
    // 检查数据大小
    if (data.isEmpty || data.length > 10 * 1024 * 1024) { // 最大 10MB
      if (kDebugMode) {
        print('Image data size invalid: ${data.length} bytes');
      }
      return false;
    }

    // 检查 JPEG 文件头 (FF D8)
    if (data.length >= 2) {
      if (data[0] == 0xFF && data[1] == 0xD8) {
        // 有效的 JPEG 开头
        return true;
      }
    }

    // 检查 PNG 文件头
    if (data.length >= 8) {
      if (data[0] == 0x89 && data[1] == 0x50 && data[2] == 0x4E && data[3] == 0x47 &&
          data[4] == 0x0D && data[5] == 0x0A && data[6] == 0x1A && data[7] == 0x0A) {
        return true;
      }
    }

    if (kDebugMode) {
      print('Invalid image header: ${data.sublist(0, 8).map((b) => b.toRadixString(16)).join(' ')}');
    }
    return false;
  }

  void _handleDisplayConfig(Map<String, dynamic> data) {
    _totalDisplays = data['total_displays'] as int;
    _currentDisplay = data['current_display'] as int;

    if (kDebugMode) {
      print('Display config updated: $_currentDisplay/$_totalDisplays');
    }

    notifyListeners();
  }

  void _handleError(dynamic error) {
    if (kDebugMode) {
      print('WebSocket error: $error');
    }
    _status = ConnectionStatus.error;
    notifyListeners();
  }

  void _handleDisconnect() {
    _status = ConnectionStatus.disconnected;
    _currentFrame = null;
    notifyListeners();
  }

  void sendSensorData(double rotationX, double rotationY, double rotationZ) {
    if (!isConnected) return;

    final message = jsonEncode({
      'type': 'SensorData',
      'rotation_x': rotationX,
      'rotation_y': rotationY,
      'rotation_z': rotationZ,
    });

    _channel?.sink.add(message);
  }

  void switchDisplay(String direction) {
    if (!isConnected) return;

    final message = jsonEncode({
      'type': 'SwitchDisplay',
      'direction': direction,
    });

    _channel?.sink.add(message);
  }

  void sendHeartbeat() {
    if (!isConnected) return;

    final message = jsonEncode({
      'type': 'Heartbeat',
    });

    _channel?.sink.add(message);
  }

  @override
  void dispose() {
    disconnect();
    super.dispose();
  }
}