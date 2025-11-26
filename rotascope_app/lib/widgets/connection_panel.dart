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
  final _formKey = GlobalKey<FormState>();

  @override
  void initState() {
    super.initState();
    final connectionService = Provider.of<ConnectionService>(context, listen: false);
    _addressController.text = connectionService.serverAddress;
  }

  @override
  Widget build(BuildContext context) {
    final connectionService = Provider.of<ConnectionService>(context);

    return SingleChildScrollView(
      padding: const EdgeInsets.all(32.0),
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          // Logo 和标题
          Container(
            padding: const EdgeInsets.all(20),
            decoration: BoxDecoration(
              gradient: LinearGradient(
                begin: Alignment.topLeft,
                end: Alignment.bottomRight,
                colors: [
                  Colors.deepPurple.shade800,
                  Colors.deepPurple.shade600,
                ],
              ),
              shape: BoxShape.circle,
            ),
            child: const Icon(
              Icons.desktop_windows,
              size: 80,
              color: Colors.white,
            ),
          ),

          const SizedBox(height: 32),

          const Text(
            'Rotascope',
            style: TextStyle(
              color: Colors.white,
              fontSize: 36,
              fontWeight: FontWeight.bold,
              letterSpacing: 1.5,
            ),
          ),

          const SizedBox(height: 8),

          const Text(
            '多屏远程显示器',
            style: TextStyle(
              color: Colors.white70,
              fontSize: 18,
              fontWeight: FontWeight.w300,
            ),
          ),

          const SizedBox(height: 32),

          const Text(
            '将手机作为PC的多个虚拟显示器\n通过旋转手机切换不同屏幕',
            textAlign: TextAlign.center,
            style: TextStyle(
              color: Colors.white70,
              fontSize: 16,
              height: 1.5,
            ),
          ),

          const SizedBox(height: 48),

          // 连接表单
          Form(
            key: _formKey,
            child: Column(
              children: [
                TextFormField(
                  controller: _addressController,
                  decoration: InputDecoration(
                    labelText: 'PC服务器地址',
                    labelStyle: const TextStyle(color: Colors.white70),
                    hintText: '192.168.1.100:8080',
                    hintStyle: const TextStyle(color: Colors.white54),
                    border: OutlineInputBorder(
                      borderRadius: BorderRadius.circular(12),
                      borderSide: const BorderSide(color: Colors.white54),
                    ),
                    enabledBorder: OutlineInputBorder(
                      borderRadius: BorderRadius.circular(12),
                      borderSide: const BorderSide(color: Colors.white54),
                    ),
                    focusedBorder: OutlineInputBorder(
                      borderRadius: BorderRadius.circular(12),
                      borderSide: const BorderSide(color: Colors.deepPurple),
                    ),
                    filled: true,
                    fillColor: Colors.white10,
                    prefixIcon: const Icon(Icons.computer, color: Colors.white70),
                  ),
                  style: const TextStyle(color: Colors.white),
                  validator: (value) {
                    if (value == null || value.isEmpty) {
                      return '请输入服务器地址';
                    }
                    if (!value.contains(':') || value.split(':').length != 2) {
                      return '格式应为 IP:端口 (如: 192.168.1.100:8080)';
                    }
                    return null;
                  },
                  onChanged: (value) {
                    connectionService.updateServerAddress(value);
                  },
                ),

                const SizedBox(height: 24),

                // 连接按钮
                SizedBox(
                  width: double.infinity,
                  child: ElevatedButton.icon(
                    onPressed: connectionService.status == ConnectionStatus.connecting
                        ? null
                        : () {
                      if (_formKey.currentState!.validate()) {
                        if (connectionService.isConnected) {
                          connectionService.disconnect();
                        } else {
                          _connectToServer(connectionService);
                        }
                      }
                    },
                    icon: connectionService.status == ConnectionStatus.connecting
                        ? const SizedBox(
                      width: 16,
                      height: 16,
                      child: CircularProgressIndicator(
                        strokeWidth: 2,
                        valueColor: AlwaysStoppedAnimation<Color>(Colors.white),
                      ),
                    )
                        : Icon(
                      connectionService.isConnected
                          ? Icons.link_off
                          : Icons.link,
                      size: 20,
                    ),
                    label: Text(
                      connectionService.status == ConnectionStatus.connecting
                          ? '连接中...'
                          : connectionService.isConnected
                          ? '断开连接'
                          : '连接到PC',
                      style: const TextStyle(fontSize: 16),
                    ),
                    style: ElevatedButton.styleFrom(
                      backgroundColor: connectionService.isConnected
                          ? Colors.red
                          : Colors.deepPurple,
                      foregroundColor: Colors.white,
                      padding: const EdgeInsets.symmetric(vertical: 16),
                      shape: RoundedRectangleBorder(
                        borderRadius: BorderRadius.circular(12),
                      ),
                      elevation: 4,
                      shadowColor: Colors.deepPurple.withOpacity(0.5),
                    ),
                  ),
                ),

                const SizedBox(height: 16),

                // 使用说明
                Container(
                  padding: const EdgeInsets.all(16),
                  decoration: BoxDecoration(
                    color: Colors.white10,
                    borderRadius: BorderRadius.circular(12),
                    border: Border.all(color: Colors.white24),
                  ),
                  child: const Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text(
                        '使用说明:',
                        style: TextStyle(
                          color: Colors.white,
                          fontSize: 14,
                          fontWeight: FontWeight.bold,
                        ),
                      ),
                      SizedBox(height: 8),
                      Text(
                        '1. 确保PC端Rotascope服务器正在运行\n'
                            '2. 输入PC的IP地址和端口号\n'
                            '3. 点击"连接到PC"建立连接\n'
                            '4. 连接成功后，左右旋转手机切换显示器\n'
                            '5. 也可以使用屏幕右下角的按钮手动切换',
                        style: TextStyle(
                          color: Colors.white70,
                          fontSize: 12,
                          height: 1.4,
                        ),
                      ),
                    ],
                  ),
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }

  Future<void> _connectToServer(ConnectionService connectionService) async {
    try {
      await connectionService.connect();

      // 连接成功提示
      if (connectionService.isConnected) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(
            content: const Text('成功连接到服务器'),
            backgroundColor: Colors.green.shade600,
            duration: const Duration(seconds: 2),
          ),
        );
      }
    } catch (e) {
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(
          content: Text('连接失败: $e'),
          backgroundColor: Colors.red.shade600,
          duration: const Duration(seconds: 3),
        ),
      );
    }
  }

  @override
  void dispose() {
    _addressController.dispose();
    super.dispose();
  }
}