import 'dart:async';

import 'package:flutter/foundation.dart';
import 'package:sensors_plus/sensors_plus.dart';

class SensorService extends ChangeNotifier {
  double _rotationX = 0;
  double _rotationY = 0;
  double _rotationZ = 0;

  bool _sensorsActive = false;
  DateTime _lastSwitchTime = DateTime.fromMillisecondsSinceEpoch(0);
  StreamSubscription<GyroscopeEvent>? _gyroscopeSubscription;

  final double _switchThreshold = 25;
  final Duration _switchCooldown = const Duration(milliseconds: 800);

  void Function(String direction)? onSwitchRequested;

  double get rotationX => _rotationX;
  double get rotationY => _rotationY;
  double get rotationZ => _rotationZ;
  bool get sensorsActive => _sensorsActive;

  void startSensors() {
    if (_sensorsActive) return;

    _gyroscopeSubscription = gyroscopeEvents.listen((event) {
      _rotationX = event.x;
      _rotationY = event.y;
      _rotationZ = event.z;

      _handleRotation();
      notifyListeners();
    });

    _sensorsActive = true;
    notifyListeners();
  }

  void stopSensors() {
    _gyroscopeSubscription?.cancel();
    _gyroscopeSubscription = null;
    _sensorsActive = false;
    notifyListeners();
  }

  void manualSwitch(String direction) {
    _requestSwitch(direction == 'previous' ? 'previous' : 'next');
  }

  void _handleRotation() {
    if (_rotationY.abs() <= _switchThreshold) return;

    final now = DateTime.now();
    if (now.difference(_lastSwitchTime) < _switchCooldown) return;

    _requestSwitch(_rotationY > 0 ? 'next' : 'previous');
  }

  void _requestSwitch(String direction) {
    _lastSwitchTime = DateTime.now();
    onSwitchRequested?.call(direction);
  }

  @override
  void dispose() {
    stopSensors();
    super.dispose();
  }
}
