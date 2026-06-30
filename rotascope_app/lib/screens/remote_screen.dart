import 'dart:async';

import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import '../services/connection_service.dart';
import '../services/sensor_service.dart';
import '../widgets/connection_panel.dart';
import '../widgets/display_hud.dart';
import '../widgets/display_view.dart';

class RemoteScreen extends StatefulWidget {
  const RemoteScreen({super.key});

  @override
  State<RemoteScreen> createState() => _RemoteScreenState();
}

class _RemoteScreenState extends State<RemoteScreen> {
  Timer? _sensorTimer;
  SensorService? _sensorService;

  @override
  void initState() {
    super.initState();

    WidgetsBinding.instance.addPostFrameCallback((_) {
      final connectionService = context.read<ConnectionService>();
      final sensorService = context.read<SensorService>();

      _sensorService = sensorService;
      sensorService.onSwitchRequested = connectionService.switchDisplay;
      sensorService.startSensors();

      _sensorTimer = Timer.periodic(const Duration(milliseconds: 150), (_) {
        connectionService.sendSensorData(
          sensorService.rotationX,
          sensorService.rotationY,
          sensorService.rotationZ,
        );
      });

      unawaited(connectionService.connect(autoReconnect: true));
    });
  }

  @override
  void dispose() {
    _sensorTimer?.cancel();
    _sensorService?.onSwitchRequested = null;
    _sensorService?.stopSensors();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final connectionService = context.watch<ConnectionService>();
    final sensorService = context.watch<SensorService>();

    return Scaffold(
      backgroundColor: Colors.black,
      body: Stack(
        fit: StackFit.expand,
        children: [
          if (connectionService.isConnected)
            const DisplayView()
          else
            const ConnectionPanel(),
          if (connectionService.isConnected)
            Positioned(
              top: MediaQuery.of(context).padding.top + 10,
              left: 12,
              right: 12,
              child: DisplayHUD(
                currentDisplay: connectionService.currentDisplay,
                totalDisplays: connectionService.totalDisplays,
                fps: connectionService.fps,
                rotation: sensorService.rotationY,
              ),
            ),
          if (connectionService.isConnected)
            Positioned(
              right: 16,
              bottom: MediaQuery.of(context).padding.bottom + 16,
              child: _DisplayControls(
                onPrevious: () => connectionService.switchDisplay('previous'),
                onNext: () => connectionService.switchDisplay('next'),
                onDisconnect: () => connectionService.disconnect(),
              ),
            ),
        ],
      ),
    );
  }
}

class _DisplayControls extends StatelessWidget {
  const _DisplayControls({
    required this.onPrevious,
    required this.onNext,
    required this.onDisconnect,
  });

  final VoidCallback onPrevious;
  final VoidCallback onNext;
  final VoidCallback onDisconnect;

  @override
  Widget build(BuildContext context) {
    return DecoratedBox(
      decoration: BoxDecoration(
        color: Colors.black.withOpacity(0.55),
        borderRadius: BorderRadius.circular(8),
        border: Border.all(color: Colors.white24),
      ),
      child: Padding(
        padding: const EdgeInsets.all(6),
        child: Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            IconButton(
              tooltip: '上一个显示器',
              onPressed: onPrevious,
              icon: const Icon(Icons.arrow_back),
            ),
            IconButton(
              tooltip: '下一个显示器',
              onPressed: onNext,
              icon: const Icon(Icons.arrow_forward),
            ),
            IconButton(
              tooltip: '断开连接',
              onPressed: onDisconnect,
              icon: const Icon(Icons.link_off),
            ),
          ],
        ),
      ),
    );
  }
}
