import 'dart:convert';
import 'dart:typed_data';
import 'package:flutter/foundation.dart';
import 'package:web_socket_channel/web_socket_channel.dart';
import 'package:web_socket_channel/io.dart';

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

  // 帧统计
  int _totalFramesReceived = 0;
  int _validFramesReceived = 0;
  int _invalidFramesReceived = 0;
  String _frameStats = '';

  ConnectionStatus get status => _status;
  String get serverAddress => _serverAddress;
  bool get isConnected => _status == ConnectionStatus.connected;
  int get currentDisplay => _currentDisplay;
  int get totalDisplays => _totalDisplays;
  Uint8List? get currentFrame => _currentFrame;
  int get frameWidth => _frameWidth;
  int get frameHeight => _frameHeight;
  String get frameStats => _frameStats;

  void updateServerAddress(String address) {
    _serverAddress = address;
    notifyListeners();
  }

  Future<void> connect() async {
    if (_status == ConnectionStatus.connected) return;

    _status = ConnectionStatus.connecting;
    notifyListeners();

    try {
      final uri = Uri.parse('ws://$_serverAddress');

      if (kDebugMode) {
        print('Connecting to WebSocket: $uri');
      }

      _channel = IOWebSocketChannel.connect(uri);

      _channel!.stream.listen(
        _handleMessage,
        onError: _handleError,
        onDone: _handleDisconnect,
      );

      _status = ConnectionStatus.connected;
      _resetFrameStats();
      notifyListeners();

      if (kDebugMode) {
        print('Successfully connected to $_serverAddress');
      }
    } catch (e) {
      _status = ConnectionStatus.error;
      notifyListeners();

      if (kDebugMode) {
        print('Connection error: $e');
      }
      rethrow;
    }
  }

  void disconnect() {
    _channel?.sink.close();
    _channel = null;
    _status = ConnectionStatus.disconnected;
    _currentFrame = null;
    notifyListeners();

    if (kDebugMode) {
      print('Disconnected from server');
    }
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
      _updateFrameStats();
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
          print('Heartbeat received from server');
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
      // 验证数据是否是有效的JPEG
      if (!_isValidImageData(message)) {
        _invalidFramesReceived++;
        _updateFrameStats();
        if (kDebugMode) {
          print('Invalid image data received: ${message.length} bytes');
        }
        return;
      }

      _validFramesReceived++;
      _currentFrame = message;
      _updateFrameStats();

      if (kDebugMode && _validFramesReceived % 30 == 0) {
        print('Frame stats: $_validFramesReceived/$_totalFramesReceived valid');
      }

      notifyListeners();
    } catch (e) {
      _invalidFramesReceived++;
      _updateFrameStats();
      if (kDebugMode) {
        print('Error handling binary message: $e');
      }
    }
  }

  bool _isValidImageData(Uint8List data) {
    // 检查数据大小
    if (data.isEmpty || data.length > 10 * 1024 * 1024) {
      return false;
    }

    // 检查 JPEG 文件头 (FF D8)
    if (data.length >= 2) {
      if (data[0] == 0xFF && data[1] == 0xD8) {
        return true;
      }
    }

    return false;
  }

  void _handleDisplayConfig(Map<String, dynamic> data) {
    _totalDisplays = (data['total_displays'] as num).toInt();
    _currentDisplay = (data['current_display'] as num).toInt();

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
    if (kDebugMode) {
      print('WebSocket connection closed');
    }
    _status = ConnectionStatus.disconnected;
    _currentFrame = null;
    notifyListeners();
  }

  void _resetFrameStats() {
    _totalFramesReceived = 0;
    _validFramesReceived = 0;
    _invalidFramesReceived = 0;
    _frameStats = '';
  }

  void _updateFrameStats() {
    final successRate = _totalFramesReceived > 0
        ? (_validFramesReceived / _totalFramesReceived * 100)
        : 0;

    _frameStats = '${_validFramesReceived}/${_totalFramesReceived} '
        '(${successRate.toStringAsFixed(1)}%)';
  }

  void sendSensorData(double rotationX, double rotationY, double rotationZ) {
    if (!isConnected) return;

    final message = jsonEncode({
      'type': 'SensorData',
      'rotation_x': rotationX,
      'rotation_y': rotationY,
      'rotation_z': rotationZ,
    });

    try {
      _channel?.sink.add(message);
    } catch (e) {
      if (kDebugMode) {
        print('Error sending sensor data: $e');
      }
    }
  }

  void switchDisplay(String direction) {
    if (!isConnected) return;

    final message = jsonEncode({
      'type': 'SwitchDisplay',
      'direction': direction,
    });

    try {
      _channel?.sink.add(message);
      if (kDebugMode) {
        print('Sent switch display command: $direction');
      }
    } catch (e) {
      if (kDebugMode) {
        print('Error sending switch command: $e');
      }
    }
  }

  @override
  void dispose() {
    disconnect();
    super.dispose();
  }
}