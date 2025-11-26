import 'package:flutter/material.dart';
import 'package:sensors_plus/sensors_plus.dart';

class SensorService extends ChangeNotifier {
  double _rotationX = 0.0;
  double _rotationY = 0.0;
  double _rotationZ = 0.0;

  bool _sensorsActive = false;
  double _lastSwitchRotation = 0.0;
  final double _switchThreshold = 25.0;

  // 防抖控制
  DateTime _lastSwitchTime = DateTime.now();
  final Duration _switchCooldown = Duration(milliseconds: 800);

  // 保存订阅以便后续取消
  dynamic _gyroscopeSubscription;
  
  // 调试标志
  final bool debugPrintEmulated = true;

  double get rotationX => _rotationX;
  double get rotationY => _rotationY;
  double get rotationZ => _rotationZ;
  bool get sensorsActive => _sensorsActive;

  void startSensors() {
    if (_sensorsActive) return;

    _gyroscopeSubscription = gyroscopeEvents.listen((GyroscopeEvent event) {
      _rotationX = event.x;
      _rotationY = event.y;
      _rotationZ = event.z;

      _handleRotation();
      notifyListeners();
    });

    _sensorsActive = true;

    if (debugPrintEmulated) {
      print('Sensors started');
    }
  }

  void stopSensors() {
    _sensorsActive = false;
    
    // 取消订阅
    _gyroscopeSubscription?.cancel();
    _gyroscopeSubscription = null;
    
    if (debugPrintEmulated) {
      print('Sensors stopped');
    }
  }

  void _handleRotation() {
    final now = DateTime.now();

    // 防抖检查
    if (now.difference(_lastSwitchTime) < _switchCooldown) {
      return;
    }

    // 检测明显的旋转变化来切换显示器
    if (_rotationY.abs() > _switchThreshold) {
      final direction = _rotationY > 0 ? 'next' : 'previous';

      // 触发切换
      _triggerSwitch(direction);
      _lastSwitchTime = now;
    }
  }

  void _triggerSwitch(String direction) {
    if (debugPrintEmulated) {
      print('Triggering display switch: $direction');
    }
  }

  // 手动触发切换（用于测试）
  void manualSwitch(String direction) {
    final now = DateTime.now();
    if (now.difference(_lastSwitchTime) >= _switchCooldown) {
      _triggerSwitch(direction);
      _lastSwitchTime = now;
    }
  }

  @override
  void dispose() {
    stopSensors();
    super.dispose();
  }
}