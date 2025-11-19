import 'dart:convert';
import 'dart:io';
import 'dart:typed_data';
import 'package:flutter/foundation.dart';

enum ConnectionStatus { disconnected, connecting, connected, error }

class ConnectionService extends ChangeNotifier {
  Socket? _socket;
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
      // 解析 address -> host:port
      final parts = _serverAddress.split(':');
      final host = parts[0];
      final port = int.tryParse(parts.length > 1 ? parts[1] : '8080') ?? 8080;

      _socket = await Socket.connect(host, port);
      // 启动读取并按 4 字节长度解析帧
      _startReading(_socket!);

      _status = ConnectionStatus.connected;
      notifyListeners();
    } catch (e) {
      _status = ConnectionStatus.error;
      notifyListeners();
      rethrow;
    }
  }

  void disconnect() {
    _socket?.destroy();
    _socket = null;
    _status = ConnectionStatus.disconnected;
    notifyListeners();
  }

  void _handleMessage(dynamic message) {
    try {
      // 保持兼容：message 已经是 Map<String, dynamic> 由 _startReading 解码后传入
      final data = message is Map<String, dynamic> ? message : jsonDecode(utf8.decode(message));
      
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
  
  void _startReading(Socket socket) {
    // 累积缓冲区
    final buffer = BytesBuilder(copy: false);
    socket.listen((Uint8List data) {
      buffer.add(data);
      var buf = buffer.takeBytes();
      var offset = 0;
      while (buf.lengthInBytes - offset >= 4) {
        final header = ByteData.sublistView(buf, offset, offset + 4);
        final len = header.getUint32(0); // big-endian 默认
        if (buf.lengthInBytes - offset - 4 < len) {
          // 不够一帧，回写剩余并等待更多数据
          buffer.add(buf.sublist(offset));
          break;
        }
        final payload = buf.sublist(offset + 4, offset + 4 + len);
        // 假设 payload 是 JSON 文本（utf8）。若为二进制，请按需处理。
        try {
          final decoded = jsonDecode(utf8.decode(payload));
          _handleMessage(decoded);
        } catch (e) {
          if (kDebugMode) print('Failed to decode frame payload: $e');
        }
        offset += 4 + len;
        if (offset == buf.lengthInBytes) {
          // 正好消费完
          break;
        }
      }
    }, onError: _handleError, onDone: _handleDisconnect, cancelOnError: true);
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
    final payload = utf8.encode(jsonEncode(message));
    _sendWithLengthPrefix(payload);
  }

  void switchDisplay(String direction) {
    if (!isConnected) return;
    
    final message = {
      'type': 'SwitchDisplay',
      'direction': direction,
    };
    final payload = utf8.encode(jsonEncode(message));
    _sendWithLengthPrefix(payload);
  }

  void _sendWithLengthPrefix(List<int> payload) {
    if (_socket == null) return;
    final header = ByteData(4);
    header.setUint32(0, payload.length); // big-endian
    _socket!.add(header.buffer.asUint8List());
    _socket!.add(payload);
  }

  @override
  void dispose() {
    disconnect();
    super.dispose();
  }
}