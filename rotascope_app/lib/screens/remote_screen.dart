import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import '../services/connection_service.dart';
import '../services/sensor_service.dart';
import '../widgets/display_view.dart';
import '../widgets/connection_panel.dart';
import '../widgets/display_hud.dart';

class RemoteScreen extends StatefulWidget {
  const RemoteScreen({super.key});

  @override
  State<RemoteScreen> createState() => _RemoteScreenState();
}

class _RemoteScreenState extends State<RemoteScreen> {
  @override
  void initState() {
    super.initState();
    
    // 启动传感器服务
    WidgetsBinding.instance.addPostFrameCallback((_) {
      final sensorService = Provider.of<SensorService>(context, listen: false);
      sensorService.startSensors();
    });
  }

  @override
  Widget build(BuildContext context) {
    final connectionService = Provider.of<ConnectionService>(context);
    final sensorService = Provider.of<SensorService>(context);

    return Scaffold(
      backgroundColor: Colors.black,
      body: Stack(
        children: [
          // 主显示区域
          if (connectionService.isConnected)
            const Center(child: DisplayView())
          else
            const Center(child: ConnectionPanel()),

          // HUD 显示当前显示器信息
          if (connectionService.isConnected)
            Positioned(
              top: 50,
              left: 0,
              right: 0,
              child: DisplayHUD(
                currentDisplay: connectionService.currentDisplay,
                totalDisplays: connectionService.totalDisplays,
                rotation: sensorService.rotationY,
              ),
            ),

          // 连接状态指示器
          Positioned(
            top: 40,
            right: 20,
            child: Container(
              padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 6),
              decoration: BoxDecoration(
                color: Colors.black54,
                borderRadius: BorderRadius.circular(20),
              ),
              child: Row(
                children: [
                  Icon(
                    connectionService.isConnected
                        ? Icons.cloud_done
                        : Icons.cloud_off,
                    color: connectionService.isConnected
                        ? Colors.green
                        : Colors.red,
                    size: 16,
                  ),
                  const SizedBox(width: 6),
                  Text(
                    connectionService.isConnected ? '已连接' : '未连接',
                    style: const TextStyle(
                      color: Colors.white,
                      fontSize: 12,
                    ),
                  ),
                ],
              ),
            ),
          ),
        ],
      ),
    );
  }
}