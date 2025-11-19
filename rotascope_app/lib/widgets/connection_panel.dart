import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import '../services/connection_service.dart';

class ConnectionPanel extends StatefulWidget {
  const ConnectionPanel({super.key});

  @override
  State<ConnectionPanel> createState() => _ConnectionPanelState();
}

class _ConnectionPanelState extends State<ConnectionPanel> {
  final TextEditingController _addressController = TextEditingController();

  @override
  void initState() {
    super.initState();
    final connectionService = Provider.of<ConnectionService>(context, listen: false);
    _addressController.text = connectionService.serverAddress;
  }

  @override
  Widget build(BuildContext context) {
    final connectionService = Provider.of<ConnectionService>(context);

    return Padding(
      padding: const EdgeInsets.all(32.0),
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          const Icon(
            Icons.desktop_windows,
            size: 80,
            color: Colors.white54,
          ),
          const SizedBox(height: 24),
          const Text(
            '多屏远程显示器',
            style: TextStyle(
              color: Colors.white,
              fontSize: 24,
              fontWeight: FontWeight.bold,
            ),
          ),
          const SizedBox(height: 8),
          const Text(
            '将手机作为PC的多个虚拟显示器\n通过旋转手机切换不同屏幕',
            textAlign: TextAlign.center,
            style: TextStyle(
              color: Colors.white70,
              fontSize: 16,
            ),
          ),
          const SizedBox(height: 32),
          TextField(
            controller: _addressController,
            decoration: InputDecoration(
              labelText: 'PC服务器地址',
              labelStyle: const TextStyle(color: Colors.white70),
              hintText: '192.168.31.169:8080',
              hintStyle: const TextStyle(color: Colors.white54),
              border: OutlineInputBorder(
                borderRadius: BorderRadius.circular(12),
                borderSide: const BorderSide(color: Colors.white54),
              ),
              enabledBorder: OutlineInputBorder(
                borderRadius: BorderRadius.circular(12),
                borderSide: const BorderSide(color: Colors.white54),
              ),
              filled: true,
              fillColor: Colors.white10,
            ),
            style: const TextStyle(color: Colors.white),
            onChanged: (value) {
              connectionService.updateServerAddress(value);
            },
          ),
          const SizedBox(height: 24),
          ElevatedButton.icon(
            onPressed: connectionService.status == ConnectionStatus.connecting
                ? null
                : () {
                    if (connectionService.isConnected) {
                      connectionService.disconnect();
                    } else {
                      connectionService.connect();
                    }
                  },
            icon: connectionService.status == ConnectionStatus.connecting
                ? const SizedBox(
                    width: 16,
                    height: 16,
                    child: CircularProgressIndicator(strokeWidth: 2),
                  )
                : Icon(
                    connectionService.isConnected
                        ? Icons.link_off
                        : Icons.link,
                  ),
            label: Text(
              connectionService.status == ConnectionStatus.connecting
                  ? '连接中...'
                  : connectionService.isConnected
                      ? '断开连接'
                      : '连接到PC',
            ),
            style: ElevatedButton.styleFrom(
              backgroundColor: connectionService.isConnected
                  ? Colors.red
                  : Colors.green,
              foregroundColor: Colors.white,
              padding: const EdgeInsets.symmetric(horizontal: 32, vertical: 16),
              shape: RoundedRectangleBorder(
                borderRadius: BorderRadius.circular(12),
              ),
            ),
          ),
        ],
      ),
    );
  }

  @override
  void dispose() {
    _addressController.dispose();
    super.dispose();
  }
}