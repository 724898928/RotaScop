import 'dart:async';
import 'dart:typed_data';

import 'package:flutter/foundation.dart';
import 'package:flutter/services.dart';

enum QuicStatus { disconnected, connecting, connected, error }

class QuicTransportService extends ChangeNotifier {
  static const _channel = MethodChannel('rotascope/quic');

  QuicStatus _status = QuicStatus.disconnected;
  String? _lastError;
  StreamSubscription<dynamic>? _frameSubscription;

  QuicStatus get status => _status;
  String? get lastError => _lastError;
  bool get isConnected => _status == QuicStatus.connected;

  final StreamController<Uint8List> _frameController =
      StreamController<Uint8List>.broadcast();

  Stream<Uint8List> get frameStream => _frameController.stream;

  Future<bool> connect(String host, int port) async {
    if (_status == QuicStatus.connected || _status == QuicStatus.connecting) {
      return false;
    }

    _status = QuicStatus.connecting;
    _lastError = null;
    notifyListeners();

    try {
      final result = await _channel.invokeMethod<bool>('connect', {
        'host': host,
        'port': port,
      });

      if (result == true) {
        _status = QuicStatus.connected;
        _channel.setMethodCallHandler(_handleNativeCall);
        notifyListeners();
        return true;
      }

      _status = QuicStatus.error;
      _lastError = 'QUIC connection rejected';
      notifyListeners();
      return false;
    } on MissingPluginException {
      _status = QuicStatus.error;
      _lastError = 'QUIC native plugin not available; use WebSocket fallback';
      if (kDebugMode) debugPrint(_lastError);
      notifyListeners();
      return false;
    } catch (e) {
      _status = QuicStatus.error;
      _lastError = e.toString();
      notifyListeners();
      return false;
    }
  }

  Future<Uint8List?> connectAndReceive(Uint8List initialData) async {
    try {
      return await _channel.invokeMethod<Uint8List>('exchange', {
        'data': initialData,
      });
    } on MissingPluginException {
      return null;
    } catch (e) {
      if (kDebugMode) debugPrint('QUIC exchange error: $e');
      return null;
    }
  }

  Future<void> disconnect() async {
    await _frameSubscription?.cancel();
    _frameSubscription = null;
    try {
      await _channel.invokeMethod<void>('disconnect');
    } catch (_) {}

    _status = QuicStatus.disconnected;
    notifyListeners();
  }

  Future<void> _handleNativeCall(MethodCall call) async {
    switch (call.method) {
      case 'onFrame':
        final data = call.arguments as Uint8List?;
        if (data != null) {
          _frameController.add(data);
        }
    }
  }

  @override
  void dispose() {
    _frameController.close();
    disconnect();
    super.dispose();
  }
}
