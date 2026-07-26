import 'dart:async';
import 'dart:convert';
import 'dart:math';

import 'package:flutter/foundation.dart';
import 'package:web_socket_channel/web_socket_channel.dart';

enum ConnectionStatus { disconnected, connecting, connected, error }

class ConnectionService extends ChangeNotifier {
  static const String defaultUsbAddress = '127.0.0.1:8083/ws';

  WebSocketChannel? _channel;
  ConnectionStatus _status = ConnectionStatus.disconnected;
  String _serverAddress = defaultUsbAddress;
  String? _lastError;

  int _currentDisplay = 0;
  int _totalDisplays = 1;

  Uint8List? _currentFrame;
  int _frameWidth = 0;
  int _frameHeight = 0;

  int _framesThisSecond = 0;
  double _fps = 0;
  int _reconnectAttempts = 0;

  bool _autoReconnect = true;
  bool _manualDisconnect = false;
  Timer? _fpsTimer;
  Timer? _heartbeatTimer;
  Timer? _reconnectTimer;

  ConnectionStatus get status => _status;
  String get serverAddress => _serverAddress;
  String? get lastError => _lastError;
  bool get isConnected => _status == ConnectionStatus.connected;
  int get currentDisplay => _currentDisplay;
  int get totalDisplays => _totalDisplays;
  Uint8List? get currentFrame => _currentFrame;
  int get frameWidth => _frameWidth;
  int get frameHeight => _frameHeight;
  double get fps => _fps;
  int get reconnectAttempts => _reconnectAttempts;

  void updateServerAddress(String address) {
    final next = address.trim();
    if (next.isEmpty || next == _serverAddress) return;

    _serverAddress = next;
    _lastError = null;
    notifyListeners();
  }

  void useUsbDefaults() {
    updateServerAddress(defaultUsbAddress);
  }

  Future<void> connect({bool autoReconnect = true}) async {
    if (_status == ConnectionStatus.connected ||
        _status == ConnectionStatus.connecting) {
      return;
    }

    _autoReconnect = autoReconnect;
    _manualDisconnect = false;
    _lastError = null;
    _status = ConnectionStatus.connecting;
    notifyListeners();

    try {
      final uri = _buildUri(_serverAddress);
      final channel = WebSocketChannel.connect(uri);
      _channel = channel;

      channel.stream.listen(
        _handleMessage,
        onError: _handleError,
        onDone: _handleDisconnect,
        cancelOnError: true,
      );

      _status = ConnectionStatus.connected;
      _reconnectAttempts = 0;
      _startTimers();
      notifyListeners();
    } catch (error) {
      _lastError = error.toString();
      _status = ConnectionStatus.error;
      notifyListeners();
      _scheduleReconnect();
      rethrow;
    }
  }

  void disconnect({bool manual = true}) {
    _manualDisconnect = manual;
    _reconnectTimer?.cancel();
    _stopTimers();
    _channel?.sink.close();
    _channel = null;
    _status = ConnectionStatus.disconnected;
    notifyListeners();
  }

  void _startTimers() {
    _fpsTimer?.cancel();
    _fpsTimer = Timer.periodic(const Duration(seconds: 1), (_) {
      _fps = _framesThisSecond.toDouble();
      _framesThisSecond = 0;
      notifyListeners();
    });

    _heartbeatTimer?.cancel();
    _heartbeatTimer = Timer.periodic(
      const Duration(seconds: 5),
      (_) => sendHeartbeat(),
    );
  }

  void _stopTimers() {
    _fpsTimer?.cancel();
    _fpsTimer = null;
    _heartbeatTimer?.cancel();
    _heartbeatTimer = null;
    _fps = 0;
    _framesThisSecond = 0;
  }

  Uri _buildUri(String address) {
    final value = address.trim();
    if (value.startsWith('ws://') || value.startsWith('wss://')) {
      return Uri.parse(value);
    }

    return Uri.parse('ws://$value');
  }

  void _handleMessage(dynamic message) {
    try {
      if (message is String) {
        _handleTextMessage(message);
      } else if (message is Uint8List) {
        _handleBinaryMessage(message);
      } else if (message is List<int>) {
        _handleBinaryMessage(Uint8List.fromList(message));
      } else if (kDebugMode) {
        debugPrint('Unknown websocket message: ${message.runtimeType}');
      }
    } catch (error) {
      _lastError = error.toString();
      if (kDebugMode) {
        debugPrint('Message handling failed: $error');
      }
      notifyListeners();
    }
  }

  void _handleTextMessage(String message) {
    final decoded = jsonDecode(message);
    if (decoded is Map<String, dynamic>) {
      _handleJsonMessage(decoded);
    }
  }

  void _handleBinaryMessage(Uint8List message) {
    if (_isImageData(message)) {
      _currentFrame = message;
      _framesThisSecond++;
      notifyListeners();
      return;
    }

    final text = utf8.decode(message, allowMalformed: false);
    _handleTextMessage(text);
  }

  void _handleJsonMessage(Map<String, dynamic> data) {
    if (data.containsKey('DisplayConfig')) {
      final config = data['DisplayConfig'];
      if (config is Map<String, dynamic>) {
        _handleDisplayConfig(config);
      }
      return;
    }

    if (data.containsKey('VideoFrame')) {
      final frame = data['VideoFrame'];
      if (frame is Map<String, dynamic>) {
        _handleVideoFrame(frame);
      }
      return;
    }

    switch (data['type']) {
      case 'DisplayConfig':
        _handleDisplayConfig(data);
      case 'VideoFrame':
        _handleVideoFrame(data);
      case 'Heartbeat':
        return;
      case 'Error':
        _lastError = data['message']?.toString() ?? 'Server error';
        notifyListeners();
    }
  }

  void _handleDisplayConfig(Map<String, dynamic> data) {
    _totalDisplays = _readInt(data['total_displays'], fallback: _totalDisplays);
    _currentDisplay = _readInt(
      data['current_display'],
      fallback: _currentDisplay,
    );
    notifyListeners();
  }

  void _handleVideoFrame(Map<String, dynamic> data) {
    final raw = data['data'];
    if (raw is! List) return;

    _currentDisplay = _readInt(
      data['display_index'],
      fallback: _currentDisplay,
    );
    _frameWidth = _readInt(data['width'], fallback: _frameWidth);
    _frameHeight = _readInt(data['height'], fallback: _frameHeight);
    _currentFrame = Uint8List.fromList(raw.cast<int>());
    _framesThisSecond++;
    notifyListeners();
  }

  int _readInt(Object? value, {required int fallback}) {
    if (value is int) return value;
    if (value is num) return value.toInt();
    if (value is String) return int.tryParse(value) ?? fallback;
    return fallback;
  }

  bool _isImageData(Uint8List data) {
    if (data.length < 4 || data.length > 20 * 1024 * 1024) return false;

    final isJpeg = data.length >= 2 && data[0] == 0xFF && data[1] == 0xD8;
    final isPng = data.length >= 8 &&
        data[0] == 0x89 &&
        data[1] == 0x50 &&
        data[2] == 0x4E &&
        data[3] == 0x47 &&
        data[4] == 0x0D &&
        data[5] == 0x0A &&
        data[6] == 0x1A &&
        data[7] == 0x0A;

    return isJpeg || isPng;
  }

  void _handleError(Object error) {
    _lastError = error.toString();
    _status = ConnectionStatus.error;
    _stopTimers();
    notifyListeners();
    _scheduleReconnect();
  }

  void _handleDisconnect() {
    _channel = null;
    _status = ConnectionStatus.disconnected;
    _stopTimers();
    notifyListeners();
    _scheduleReconnect();
  }

  void _scheduleReconnect() {
    if (!_autoReconnect || _manualDisconnect) return;

    _reconnectTimer?.cancel();
    _reconnectAttempts++;

    final seconds = min(30, 1 << min(_reconnectAttempts, 5));
    _reconnectTimer = Timer(Duration(seconds: seconds), () {
      if (_manualDisconnect) return;
      connect(autoReconnect: true);
    });
  }

  void sendSensorData(double rotationX, double rotationY, double rotationZ) {
    if (!isConnected) return;

    _channel?.sink.add(jsonEncode({
      'type': 'SensorData',
      'rotation_x': rotationX,
      'rotation_y': rotationY,
      'rotation_z': rotationZ,
    }));
  }

  void switchDisplay(String direction) {
    if (!isConnected) return;

    final normalized = direction == 'previous' ? 'previous' : 'next';
    _channel?.sink.add(jsonEncode({
      'type': 'SwitchDisplay',
      'direction': normalized,
    }));
  }

  void sendTouchEvent(String eventType, double normalizedX, double normalizedY) {
    if (!isConnected) return;

    _channel?.sink.add(jsonEncode({
      'type': 'TouchEvent',
      'event': eventType,
      'x': normalizedX,
      'y': normalizedY,
    }));
  }

  void sendHeartbeat() {
    if (!isConnected) return;

    _channel?.sink.add(jsonEncode({
      'type': 'Heartbeat',
    }));
  }

  @override
  void dispose() {
    disconnect();
    super.dispose();
  }
}
