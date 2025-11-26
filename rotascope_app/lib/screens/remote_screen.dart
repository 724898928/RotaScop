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
            const DisplayView()
          else
            const ConnectionPanel(),

          // HUD 显示当前显示器信息和连接状态
          // 修复：使用 Padding 替代 margin
          Positioned(
            top: MediaQuery.of(context).padding.top + 10,
            left: 0,
            right: 0,
            child: Padding(
              padding: const EdgeInsets.symmetric(horizontal: 20),
              child: Column(
                children: [
                  if (connectionService.isConnected)
                    DisplayHUD(
                      currentDisplay: connectionService.currentDisplay,
                      totalDisplays: connectionService.totalDisplays,
                      rotation: sensorService.rotationY,
                    ),

                  const SizedBox(height: 10),

                  // 连接状态指示器
                  Container(
                    padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
                    decoration: BoxDecoration(
                      color: Colors.black.withOpacity(0.7),
                      borderRadius: BorderRadius.circular(20),
                      border: Border.all(
                        color: connectionService.isConnected
                            ? Colors.green
                            : Colors.red,
                        width: 1,
                      ),
                    ),
                    child: Row(
                      mainAxisSize: MainAxisSize.min,
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
                        const SizedBox(width: 8),
                        Text(
                          connectionService.isConnected
                              ? '已连接到 ${connectionService.serverAddress}'
                              : '未连接',
                          style: const TextStyle(
                            color: Colors.white,
                            fontSize: 14,
                            fontWeight: FontWeight.w500,
                          ),
                        ),
                        if (connectionService.status == ConnectionStatus.connecting)
                          const SizedBox(width: 8),
                        if (connectionService.status == ConnectionStatus.connecting)
                          const SizedBox(
                            width: 12,
                            height: 12,
                            child: CircularProgressIndicator(
                              strokeWidth: 2,
                              valueColor: AlwaysStoppedAnimation<Color>(Colors.blue),
                            ),
                          ),
                      ],
                    ),
                  ),
                ],
              ),
            ),
          ),

          // 控制按钮
          if (connectionService.isConnected)
            Positioned(
              bottom: 20,
              right: 20,
              child: Row(
                children: [
                  FloatingActionButton(
                    onPressed: () {
                      connectionService.switchDisplay('previous');
                    },
                    backgroundColor: Colors.deepPurple,
                    mini: true,
                    child: const Icon(Icons.arrow_back, color: Colors.white),
                  ),
                  const SizedBox(width: 10),
                  FloatingActionButton(
                    onPressed: () {
                      connectionService.switchDisplay('next');
                    },
                    backgroundColor: Colors.deepPurple,
                    mini: true,
                    child: const Icon(Icons.arrow_forward, color: Colors.white),
                  ),
                ],
              ),
            ),
        ],
      ),
    );
  }
}