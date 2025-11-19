import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:sensors_plus/sensors_plus.dart';

class SensorService extends ChangeNotifier {
  double _rotationX = 0.0;
  double _rotationY = 0.0;
  double _rotationZ = 0.0;
  
  bool _sensorsActive = false;
  double _lastSwitchRotation = 0.0;
  final double _switchThreshold = 25.0;

  double get rotationX => _rotationX;
  double get rotationY => _rotationY;
  double get rotationZ => _rotationZ;
  bool get sensorsActive => _sensorsActive;

  void startSensors() {
    if (_sensorsActive) return;
    
    gyroscopeEvents.listen((GyroscopeEvent event) {
      _rotationX = event.x;
      _rotationY = event.y;
      _rotationZ = event.z;
      
      _handleRotation();
      notifyListeners();
    });
    
    _sensorsActive = true;
  }

  void stopSensors() {
    _sensorsActive = false;
  }

  void _handleRotation() {
    // 检测明显的旋转变化来切换显示器
    if (_rotationY.abs() > _switchThreshold) {
      final now = DateTime.now().millisecondsSinceEpoch;
      
      // 防抖处理，避免频繁切换
      if (now - _lastSwitchRotation > 1000) {
        if (_rotationY > 0) {
          _triggerSwitch('next');
        } else {
          _triggerSwitch('previous');
        }
        _lastSwitchRotation =  now.toDouble();
      }
    }
  }

  void _triggerSwitch(String direction) {
    // 这里可以通过回调或事件总线通知连接服务
    // 在实际实现中，可以使用 Provider 或事件系统
    if (kDebugMode) {
      print('Switching display: $direction');
    }
  }

  @override
  void dispose() {
    stopSensors();
    super.dispose();
  }
}