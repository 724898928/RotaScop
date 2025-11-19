import 'dart:convert';
import 'package:flutter/foundation.dart';
import 'package:web_socket_channel/web_socket_channel.dart';

enum ConnectionStatus { disconnected, connecting, connected, error }

class ConnectionService extends ChangeNotifier {
  WebSocketChannel? _channel;
  ConnectionStatus _status = ConnectionStatus.disconnected;
  String _serverAddress = '192.168.31.169:8080';
  
  int _currentDisplay = 0;
  int _totalDisplays = 3;
  List<Map<String, dynamic>> _displayResolutions = [];
  
  List<int>? _currentFrame;
  int _frameWidth = 1920;
  int _frameHeight = 1080;

  ConnectionStatus get status => _status;
  String get serverAddress => _serverAddress;
  bool get isConnected => _status == ConnectionStatus.connected;
  int get currentDisplay => _currentDisplay;
  int get totalDisplays => _totalDisplays;
  List<int>? get currentFrame => _currentFrame;
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
    notifyListeners();
  }

  void _handleMessage(dynamic message) {
    try {
      final data = jsonDecode(utf8.decode(message));
      
      if (data['type'] == 'VideoFrame') {
        _handleVideoFrame(data);
      } else if (data['type'] == 'DisplayConfig') {
        _handleDisplayConfig(data);
      }
    } catch (e) {
      if (kDebugMode) {
        print('Error handling message: $e');
      }
    }
  }

  void _handleVideoFrame(Map<String, dynamic> data) {
    final displayIndex = data['display_index'] as int;
    final width = data['width'] as int;
    final height = data['height'] as int;
    final jpegData = List<int>.from(data['data'] as List);
    
    _currentDisplay = displayIndex;
    _frameWidth = width;
    _frameHeight = height;
    _currentFrame = jpegData;
    
    notifyListeners();
  }

  void _handleDisplayConfig(Map<String, dynamic> data) {
    _totalDisplays = data['total_displays'] as int;
    _currentDisplay = data['current_display'] as int;
    
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
    notifyListeners();
  }

  void sendSensorData(double rotationX, double rotationY, double rotationZ) {
    if (!isConnected) return;
    
    final message = {
      'type': 'SensorData',
      'rotation_x': rotationX,
      'rotation_y': rotationY,
      'rotation_z': rotationZ,
    };
    
    _channel?.sink.add(jsonEncode(message));
  }

  void switchDisplay(String direction) {
    if (!isConnected) return;
    
    final message = {
      'type': 'SwitchDisplay',
      'direction': direction,
    };
    
    _channel?.sink.add(jsonEncode(message));
  }

  @override
  void dispose() {
    disconnect();
    super.dispose();
  }
}